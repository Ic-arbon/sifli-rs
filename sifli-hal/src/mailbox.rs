//! Mailbox HAL driver
//!
//! The Mailbox provides hardware-assisted inter-processor communication between
//! HCPU and LCPU subsystems on SF32LB52x chips.
//!
//! ## Hardware Architecture
//!
//! - Two independent mailbox peripherals:
//!   - `MAILBOX1`: HCPU transmits → LCPU receives
//!   - `MAILBOX2`: LCPU transmits → HCPU receives (not yet in PAC)
//! - Each mailbox has 4 channels
//! - Each channel has 16 independent interrupt bits (0-15)
//! - Mutex functionality via exclusive registers (CxEXR)
//!
//! ## Key Design Principle
//!
//! MAILBOX1 and MAILBOX2 are **unidirectional**. On HCPU:
//! - MAILBOX1 is transmit-only (no interrupt)
//! - MAILBOX2 is receive-only (with interrupt)
//!
//! Therefore we have two separate types: `TxMailbox` and `RxMailbox`.
//!
//! ## Basic Usage
//!
//! ### Transmit (HCPU side with MAILBOX1)
//!
//! ```no_run
//! # use sifli_hal::{mailbox, bind_interrupts, peripherals};
//! # let p = unsafe { peripherals::Peripherals::steal() };
//! // Create transmit mailbox
//! let mut tx = mailbox::TxMailbox::new(p.MAILBOX1);
//!
//! // Trigger interrupt on LCPU side
//! tx.trigger(mailbox::Channel::CH0, 0); // trigger bit 0 on channel 0
//! tx.trigger(mailbox::Channel::CH1, 5); // trigger bit 5 on channel 1
//!
//! // Or trigger multiple bits at once
//! tx.trigger_mask(mailbox::Channel::CH0, 0b1111); // trigger bits 0-3
//! ```
//!
//! ### Receive (HCPU side with MAILBOX2, when available in PAC)
//!
//! ```no_run,ignore
//! # use sifli_hal::{mailbox, bind_interrupts, peripherals};
//! # let p = unsafe { peripherals::Peripherals::steal() };
//! bind_interrupts!(struct Irqs {
//!     MAILBOX2_CH1 => mailbox::InterruptHandler<peripherals::MAILBOX2>;
//! });
//!
//! let mut rx = mailbox::RxMailbox::new(p.MAILBOX2, Irqs);
//!
//! // Enable specific interrupt bits
//! rx.enable_channel(mailbox::Channel::CH0, 0);
//!
//! // In interrupt handler or polling:
//! if rx.is_pending(mailbox::Channel::CH0, 0) {
//!     // Handle interrupt
//!     rx.clear(mailbox::Channel::CH0, 0);
//! }
//! ```
//!
//! ## Mutex Usage
//!
//! Mailbox channels can be used as hardware mutex to synchronize access to
//! shared resources between cores:
//!
//! ```no_run
//! # use sifli_hal::{mailbox, peripherals};
//! # let p = unsafe { peripherals::Peripherals::steal() };
//! use mailbox::MutexOps;
//!
//! let tx = mailbox::TxMailbox::new(p.MAILBOX1);
//!
//! // Try to acquire lock
//! match TxMailbox::<_>::lock(mailbox::Channel::CH0) {
//!     mailbox::LockCore::Unlocked => {
//!         // Lock acquired, do critical work
//!         // ...
//!         unsafe { TxMailbox::<_>::unlock(mailbox::Channel::CH0) };
//!     }
//!     core => {
//!         // Lock held by another core
//!         println!("Lock held by {:?}", core);
//!     }
//! }
//! ```
//!
//! ## Implementation Notes
//!
//! - Currently only MAILBOX1 is supported (transmit-only on HCPU)
//! - MAILBOX2 support will be added when it's available in sifli-pac
//! - The C SDK uses a single struct with irqn polarity to distinguish TX/RX,
//!   but this Rust implementation uses separate types for better type safety
//! - No state machine overhead - direct register access only


use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::interrupt;
use crate::interrupt::typelevel::Interrupt as _;
use crate::pac::mailbox::Mailbox1;
use crate::rcc::RccEnableReset;

/// Mailbox channel identifier (0-3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel(u8);

impl Channel {
    /// Create a new channel (0-3)
    ///
    /// Returns `None` if the channel number is >= 4
    #[inline]
    pub const fn new(channel: u8) -> Option<Self> {
        if channel < 4 {
            Some(Channel(channel))
        } else {
            None
        }
    }

    /// Create a new channel without bounds checking
    ///
    /// # Safety
    /// The caller must ensure that `channel` is in range 0-3
    #[inline]
    pub const unsafe fn new_unchecked(channel: u8) -> Self {
        Channel(channel)
    }

    /// Get channel index
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl Channel {
    /// Channel 0
    pub const CH0: Self = Channel(0);
    /// Channel 1
    pub const CH1: Self = Channel(1);
    /// Channel 2
    pub const CH2: Self = Channel(2);
    /// Channel 3
    pub const CH3: Self = Channel(3);
}

/// Mutex lock core identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LockCore {
    /// Mutex is not locked
    Unlocked = 0,
    /// Mutex is locked by HCPU
    Hcpu = 1,
    /// Mutex is locked by LCPU
    Lcpu = 2,
    /// Mutex is locked by BCPU
    Bcpu = 3,
}

impl LockCore {
    /// Convert from register value
    pub(crate) fn from_bits(value: u8) -> Self {
        match value & 0x3 {
            0 => LockCore::Unlocked,
            1 => LockCore::Hcpu,
            2 => LockCore::Lcpu,
            3 => LockCore::Bcpu,
            _ => unreachable!(),
        }
    }

    /// Check if the mutex is locked
    #[inline]
    pub const fn is_locked(self) -> bool {
        !matches!(self, LockCore::Unlocked)
    }
}

/// Transmit-only mailbox (e.g., MAILBOX1 on HCPU)
pub struct TxMailbox<'d, T: TxInstance> {
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: TxInstance> TxMailbox<'d, T> {
    /// Create a new transmit mailbox
    pub fn new(peri: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(peri);
        crate::rcc::enable_and_reset::<T>();
        Self { _peri: peri }
    }

    /// Trigger an interrupt on the specified channel and bit
    ///
    /// # Arguments
    /// - `channel`: Channel 0-3
    /// - `bit`: Interrupt bit 0-15
    #[inline]
    pub fn trigger(&mut self, channel: Channel, bit: u8) {
        assert!(bit < 16, "bit must be 0-15");
        let regs = T::regs();
        let idx = channel.index();

        // Write 1 to ITR register to trigger interrupt on remote side
        regs.itr(idx).write(|w| w.set_int(bit as usize, true));
    }

    /// Trigger multiple bits at once on a channel
    ///
    /// # Arguments
    /// - `channel`: Channel 0-3
    /// - `mask`: 16-bit mask where each set bit triggers that interrupt
    #[inline]
    pub fn trigger_mask(&mut self, channel: Channel, mask: u16) {
        let regs = T::regs();
        let idx = channel.index();

        regs.itr(idx).write(|w| w.0 = mask as u32);
    }
}

/// Receive-only mailbox with interrupt support (e.g., MAILBOX2 on HCPU)
pub struct RxMailbox<'d, T: RxInstance> {
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: RxInstance> RxMailbox<'d, T> {
    /// Create a new receive mailbox with interrupt handler
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        into_ref!(peri);
        crate::rcc::enable_and_reset::<T>();

        // Enable interrupt
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { _peri: peri }
    }

    /// Enable interrupt for a specific channel and bit
    ///
    /// # Arguments
    /// - `channel`: Channel 0-3
    /// - `bit`: Interrupt bit 0-15
    #[inline]
    pub fn enable_channel(&mut self, channel: Channel, bit: u8) {
        assert!(bit < 16, "bit must be 0-15");
        let regs = T::regs();
        let idx = channel.index();

        regs.ier(idx).modify(|w| w.set_int(bit as usize, true));
    }

    /// Disable interrupt for a specific channel and bit
    #[inline]
    pub fn disable_channel(&mut self, channel: Channel, bit: u8) {
        assert!(bit < 16, "bit must be 0-15");
        let regs = T::regs();
        let idx = channel.index();

        regs.ier(idx).modify(|w| w.set_int(bit as usize, false));
    }

    /// Enable multiple bits at once on a channel
    ///
    /// # Arguments
    /// - `channel`: Channel 0-3
    /// - `mask`: 16-bit mask where each set bit enables that interrupt
    #[inline]
    pub fn enable_mask(&mut self, channel: Channel, mask: u16) {
        let regs = T::regs();
        let idx = channel.index();

        regs.ier(idx).modify(|w| w.0 |= mask as u32);
    }

    /// Disable multiple bits at once on a channel
    #[inline]
    pub fn disable_mask(&mut self, channel: Channel, mask: u16) {
        let regs = T::regs();
        let idx = channel.index();

        regs.ier(idx).modify(|w| w.0 &= !(mask as u32));
    }

    /// Check if a specific interrupt is pending
    #[inline]
    pub fn is_pending(&self, channel: Channel, bit: u8) -> bool {
        assert!(bit < 16, "bit must be 0-15");
        let regs = T::regs();
        let idx = channel.index();

        regs.misr(idx).read().int(bit as usize)
    }

    /// Get the masked interrupt status for a channel (all 16 bits)
    #[inline]
    pub fn pending_mask(&self, channel: Channel) -> u16 {
        let regs = T::regs();
        let idx = channel.index();

        (regs.misr(idx).read().0 & 0xFFFF) as u16
    }

    /// Clear a specific interrupt
    #[inline]
    pub fn clear(&mut self, channel: Channel, bit: u8) {
        assert!(bit < 16, "bit must be 0-15");
        let regs = T::regs();
        let idx = channel.index();

        regs.icr(idx).write(|w| w.set_int(bit as usize, true));
    }

    /// Clear multiple interrupts at once
    #[inline]
    pub fn clear_mask(&mut self, channel: Channel, mask: u16) {
        let regs = T::regs();
        let idx = channel.index();

        regs.icr(idx).write(|w| w.0 = mask as u32);
    }
}

/// Mutex operations (available on both TX and RX mailboxes)
pub trait MutexOps {
    /// Get the mailbox registers
    fn regs() -> Mailbox1;

    /// Try to lock a mutex channel
    ///
    /// Returns `LockCore::Unlocked` if lock was acquired,
    /// otherwise returns which core currently holds the lock.
    #[inline]
    fn lock(channel: Channel) -> LockCore {
        let regs = Self::regs();
        let idx = channel.index();

        // Read EXR register - hardware automatically locks if available
        let exr = regs.exr(idx).read();

        // Check EX bit (bit 31) - if set, lock was acquired
        if exr.ex() {
            LockCore::Unlocked
        } else {
            // Lock held by another core, return the ID
            LockCore::from_bits(exr.id())
        }
    }

    /// Unlock a mutex channel
    ///
    /// # Safety
    /// Caller must ensure they own the lock
    #[inline]
    unsafe fn unlock(channel: Channel) {
        let regs = Self::regs();
        let idx = channel.index();

        // Write EX bit to unlock
        regs.exr(idx).write(|w| w.set_ex(true));
    }

    /// Check who holds the lock without attempting to acquire
    #[inline]
    fn lock_status(channel: Channel) -> LockCore {
        let regs = Self::regs();
        let idx = channel.index();

        let exr = regs.exr(idx).read();

        if exr.ex() {
            LockCore::Unlocked
        } else {
            LockCore::from_bits(exr.id())
        }
    }
}

impl<'d, T: TxInstance> MutexOps for TxMailbox<'d, T> {
    fn regs() -> Mailbox1 {
        T::regs()
    }
}

impl<'d, T: RxInstance> MutexOps for RxMailbox<'d, T> {
    fn regs() -> Mailbox1 {
        T::regs()
    }
}

/// Transmit mailbox instance trait
trait SealedTxInstance: RccEnableReset {
    fn regs() -> Mailbox1;
}

/// Transmit mailbox instance trait
#[allow(private_bounds)]
pub trait TxInstance: SealedTxInstance + Peripheral<P = Self> + 'static {}

/// Receive mailbox instance trait
trait SealedRxInstance: RccEnableReset {
    fn regs() -> Mailbox1;
}

/// Receive mailbox instance trait
#[allow(private_bounds)]
pub trait RxInstance: SealedRxInstance + Peripheral<P = Self> + 'static {
    /// Interrupt for this mailbox
    type Interrupt: interrupt::typelevel::Interrupt;
}

/// Interrupt handler for receive mailbox
pub struct InterruptHandler<T: RxInstance> {
    _phantom: PhantomData<T>,
}

impl<T: RxInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();

        // Process all channels
        for ch in 0..4 {
            let misr = regs.misr(ch).read().0 as u16;
            if misr != 0 {
                // Clear all pending interrupts for this channel
                regs.icr(ch).write(|w| w.0 = misr as u32);

                // TODO: Wake user tasks based on channel/bit
                // This requires async support with waitqueues
            }
        }
    }
}

// Peripheral implementations
// These should eventually be moved to _generated.rs

use crate::peripherals;

// MAILBOX1: TX on HCPU, RX on LCPU
impl SealedTxInstance for peripherals::MAILBOX1 {
    fn regs() -> Mailbox1 {
        crate::pac::MAILBOX1
    }
}

impl TxInstance for peripherals::MAILBOX1 {}

// TODO: Add MAILBOX2 when it's available in PAC
// MAILBOX2: RX on HCPU (with interrupt), TX on LCPU
// impl SealedRxInstance for peripherals::MAILBOX2 {
//     fn regs() -> Mailbox1 {
//         crate::pac::MAILBOX2
//     }
// }
//
// impl RxInstance for peripherals::MAILBOX2 {
//     // MAILBOX2_CH1 is the receive interrupt on HCPU
//     type Interrupt = crate::interrupt::typelevel::MAILBOX2_CH1;
// }
