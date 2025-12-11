//! Mailbox HAL driver
//!
//! Provides hardware mailbox for inter-processor communication on SF32LB52x chips.
//!
//! ## Hardware Architecture
//!
//! Physical MAILBOX peripherals on chip:
//! - **MAILBOX1** @ 0x50082000 (HPSYS address space) - 4 channels
//! - **MAILBOX2** @ 0x40042000 (LPSYS address space) - 2 channels
//!
//! Each CPU uses one for TX, listens to the other's IRQ for RX:
//!
//! - **HCPU usage**:
//!   - TX: Write MAILBOX1.ITR → triggers LCPU interrupt
//!   - RX: Handle MAILBOX2_CH1_IRQn → read LCPU shared memory
//!     (HCPU doesn't need to access MAILBOX2 registers)
//!
//! - **LCPU usage**:
//!   - TX: Write MAILBOX2.ITR → triggers HCPU interrupt
//!   - RX: Handle MAILBOX1 interrupts → read HCPU shared memory
//!
//! ## Design: Channels as Fields
//!
//! Each mailbox exposes channels as struct fields for compile-time safety:
//! - `Mailbox<MAILBOX1>` has 4 channels: `ch1`, `ch2`, `ch3`, `ch4`
//! - `Mailbox<MAILBOX2>` has 2 channels: `ch1`, `ch2`
//!
//! This design provides:
//! - **Compile-time safety**: Cannot access non-existent channels
//! - **Zero-cost abstractions**: Channel ownership can be split across tasks
//! - **Type-driven API**: Different mailbox instances have different field counts
//!
//! ## Usage
//!
//! ```no_run
//! use sifli_hal::{mailbox, peripherals};
//!
//! let p = /* get peripherals */;
//! # unsafe { peripherals::Peripherals::steal() };
//!
//! let mut mb = mailbox::Mailbox1::new(p.MAILBOX1);
//!
//! // Access channels as fields - compile-time safe!
//! mb.ch1.trigger(0);  // Trigger interrupt bit 0 on channel 1
//! mb.ch4.trigger(15); // Trigger interrupt bit 15 on channel 4
//!
//! // Use as mutex (all cores can access)
//! match mb.ch1.try_lock() {
//!     mailbox::LockCore::Unlocked => {
//!         // Got the lock
//!         // ... critical section ...
//!         unsafe { mb.ch1.unlock() };
//!     }
//!     core => defmt::info!("Locked by {:?}", core),
//! }
//!
//! // Split ownership across tasks
//! let (ch1, ch2, ch3, ch4) = mb.split();
//! spawner.spawn(task1(ch1)).unwrap();
//! spawner.spawn(task2(ch2)).unwrap();
//! ```

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use crate::peripherals;

/// Re-export LockCore enum from PAC
pub use crate::pac::mailbox::vals::LockCore;

pub trait LockCoreExt {
    fn is_locked(&self) -> bool;
}

/// Extension methods for LockCore
impl LockCoreExt for LockCore {
    /// Check if locked
    #[inline]
    fn is_locked(&self) -> bool {
        !matches!(self, LockCore::Unlocked)
    }
}

/// Sealed trait to constrain mailbox peripheral types
mod sealed {
    pub trait SealedMailboxInstance {}
}

// Type aliases for register types
type IxrReg = crate::pac::common::Reg<crate::pac::mailbox::regs::Ixr, crate::pac::common::RW>;
type ExrReg = crate::pac::common::Reg<crate::pac::mailbox::regs::Exr, crate::pac::common::RW>;

/// Trait for mailbox peripheral instances
///
/// This trait is sealed and cannot be implemented outside this module.
/// It provides a unified interface to access MAILBOX1 and MAILBOX2 registers.
pub trait MailboxInstance: sealed::SealedMailboxInstance + 'static {
    /// Get IER register
    fn ier(ch: usize) -> IxrReg;
    /// Get ITR register
    fn itr(ch: usize) -> IxrReg;
    /// Get ICR register
    fn icr(ch: usize) -> IxrReg;
    /// Get ISR register
    fn isr(ch: usize) -> IxrReg;
    /// Get MISR register
    fn misr(ch: usize) -> IxrReg;
    /// Get EXR register
    fn exr(ch: usize) -> ExrReg;
}

impl sealed::SealedMailboxInstance for peripherals::MAILBOX1 {}
impl MailboxInstance for peripherals::MAILBOX1 {
    #[inline]
    fn ier(ch: usize) -> IxrReg {
        crate::pac::MAILBOX1.ier(ch)
    }
    #[inline]
    fn itr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX1.itr(ch)
    }
    #[inline]
    fn icr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX1.icr(ch)
    }
    #[inline]
    fn isr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX1.isr(ch)
    }
    #[inline]
    fn misr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX1.misr(ch)
    }
    #[inline]
    fn exr(ch: usize) -> ExrReg {
        crate::pac::MAILBOX1.exr(ch)
    }
}

impl sealed::SealedMailboxInstance for peripherals::MAILBOX2 {}
impl MailboxInstance for peripherals::MAILBOX2 {
    #[inline]
    fn ier(ch: usize) -> IxrReg {
        crate::pac::MAILBOX2.ier(ch)
    }
    #[inline]
    fn itr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX2.itr(ch)
    }
    #[inline]
    fn icr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX2.icr(ch)
    }
    #[inline]
    fn isr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX2.isr(ch)
    }
    #[inline]
    fn misr(ch: usize) -> IxrReg {
        crate::pac::MAILBOX2.misr(ch)
    }
    #[inline]
    fn exr(ch: usize) -> ExrReg {
        crate::pac::MAILBOX2.exr(ch)
    }
}

/// Mailbox channel with generic peripheral type
///
/// Generic over:
/// - `T`: Mailbox peripheral type (MAILBOX1 or MAILBOX2)
/// - `CH`: Channel index (compile-time constant)
pub struct MailboxChannel<'d, T: MailboxInstance, const CH: usize> {
    _phantom: core::marker::PhantomData<&'d mut T>,
}

impl<'d, T: MailboxInstance, const CH: usize> MailboxChannel<'d, T, CH> {
    /// Create new channel (internal use only)
    #[inline]
    const fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    /// Trigger interrupt on remote core
    ///
    /// # Arguments
    /// - `bit`: Interrupt bit 0-15
    #[inline]
    pub fn trigger(&mut self, bit: u8) {
        assert!(bit < 16, "bit must be 0-15");
        T::itr(CH).write(|w| w.set_int(bit as usize, true));
    }

    /// Trigger multiple bits at once
    ///
    /// # Arguments
    /// - `mask`: Bitmask of interrupts to trigger (bits 0-15)
    ///
    /// # Example
    /// ```no_run
    /// # let mut mb = todo!();
    /// // Trigger bits 0, 3, and 7 simultaneously
    /// mb.ch1.trigger_mask(0b1000_1001);
    /// ```
    #[inline]
    pub fn trigger_mask(&mut self, mask: u16) {
        T::itr(CH).write(|w| w.0 = mask as u32);
    }

    /// Enable interrupt reception (unmask)
    ///
    /// Allows receiving interrupts from remote core for this bit.
    /// Must be called on the **receiving side** before remote can send interrupts.
    ///
    /// # Arguments
    /// - `bit`: Interrupt bit 0-15
    #[inline]
    pub fn enable_interrupt(&mut self, bit: u8) {
        assert!(bit < 16, "bit must be 0-15");
        T::ier(CH).modify(|w| w.set_int(bit as usize, true));
    }

    /// Disable interrupt reception (mask)
    ///
    /// Prevents receiving interrupts from remote core for this bit.
    ///
    /// # Arguments
    /// - `bit`: Interrupt bit 0-15
    #[inline]
    pub fn disable_interrupt(&mut self, bit: u8) {
        assert!(bit < 16, "bit must be 0-15");
        T::ier(CH).modify(|w| w.set_int(bit as usize, false));
    }

    /// Try to acquire mutex lock
    ///
    /// Returns `Unlocked` if lock was acquired, otherwise returns current owner.
    ///
    /// # Hardware behavior
    /// Reading the EXR register is an atomic operation:
    /// - If unlocked: Sets the lock and returns `Unlocked`
    /// - If locked: Returns the core ID that owns the lock
    /// - Read-to-clear: When `EX = 1` is read, hardware clears it to `EX = 0` to claim the mutex
    ///
    /// # Example
    /// ```no_run
    /// # let mut mb = todo!();
    /// match mb.ch1.try_lock() {
    ///     LockCore::Unlocked => {
    ///         // Lock acquired, enter critical section
    ///         critical_work();
    ///         unsafe { mb.ch1.unlock() };
    ///     }
    ///     LockCore::Hcpu => {
    ///         // Locked by HCPU, retry later
    ///     }
    ///     _ => {}
    /// }
    /// ```
    #[inline]
    pub fn try_lock(&mut self) -> LockCore {
        let exr = T::exr(CH).read();

        if exr.ex() {
            LockCore::Unlocked
        } else {
            exr.id()
        }
    }

    /// Unlock mutex
    ///
    /// # Safety
    /// Caller must own the lock (i.e., `try_lock()` returned `Unlocked`)
    #[inline]
    pub unsafe fn unlock(&mut self) {
        T::exr(CH).write(|w| w.set_ex(true));
    }
}

/// MAILBOX1 driver (4 channels)
pub struct Mailbox1<'d> {
    _peri: PeripheralRef<'d, peripherals::MAILBOX1>,
    /// Channel 1 (hardware channel 0)
    pub ch1: MailboxChannel<'d, peripherals::MAILBOX1, 0>,
    pub ch2: MailboxChannel<'d, peripherals::MAILBOX1, 1>,
    pub ch3: MailboxChannel<'d, peripherals::MAILBOX1, 2>,
    pub ch4: MailboxChannel<'d, peripherals::MAILBOX1, 3>,
}

impl<'d> Mailbox1<'d> {
    /// Create new MAILBOX1 instance
    ///
    /// Enables the mailbox peripheral clock via RCC.
    pub fn new(peri: impl Peripheral<P = peripherals::MAILBOX1> + 'd) -> Self {
        into_ref!(peri);
        crate::rcc::enable_and_reset::<peripherals::MAILBOX1>();
        Self {
            _peri: peri,
            ch1: MailboxChannel::new(),
            ch2: MailboxChannel::new(),
            ch3: MailboxChannel::new(),
            ch4: MailboxChannel::new(),
        }
    }

    /// Split into individual channels for separate ownership
    ///
    /// This allows passing channels to different tasks or modules.
    ///
    /// # Example
    /// ```no_run
    /// # use embassy_executor::Spawner;
    /// # async fn example(spawner: Spawner) {
    /// let p = sifli_hal::init(Default::default());
    /// let mb = sifli_hal::mailbox::Mailbox1::new(p.MAILBOX1);
    ///
    /// let (ch1, ch2, ch3, ch4) = mb.split();
    ///
    /// spawner.spawn(control_task(ch1)).unwrap();
    /// spawner.spawn(data_task(ch2)).unwrap();
    /// # }
    /// ```
    pub fn split(
        self,
    ) -> (
        MailboxChannel<'d, peripherals::MAILBOX1, 0>,
        MailboxChannel<'d, peripherals::MAILBOX1, 1>,
        MailboxChannel<'d, peripherals::MAILBOX1, 2>,
        MailboxChannel<'d, peripherals::MAILBOX1, 3>,
    ) {
        (self.ch1, self.ch2, self.ch3, self.ch4)
    }
}

/// MAILBOX2 driver (2 channels)
pub struct Mailbox2<'d> {
    _peri: PeripheralRef<'d, peripherals::MAILBOX2>,
    pub ch1: MailboxChannel<'d, peripherals::MAILBOX2, 0>,
    pub ch2: MailboxChannel<'d, peripherals::MAILBOX2, 1>,
}

impl<'d> Mailbox2<'d> {
    /// Create new MAILBOX2 instance
    ///
    /// Note: MAILBOX2 is in LPSYS and clock is managed by LPSYS_RCC, not HPSYS_RCC.
    /// The clock should already be enabled by bootloader or LCPU firmware.
    pub fn new(peri: impl Peripheral<P = peripherals::MAILBOX2> + 'd) -> Self {
        into_ref!(peri);
        // MAILBOX2 is in LPSYS, no HPSYS RCC control
        Self {
            _peri: peri,
            ch1: MailboxChannel::new(),
            ch2: MailboxChannel::new(),
        }
    }

    /// Split into individual channels for separate ownership
    ///
    /// This allows passing channels to different tasks or modules.
    ///
    /// # Example
    /// ```no_run
    /// # use embassy_executor::Spawner;
    /// # async fn example(spawner: Spawner) {
    /// let p = sifli_hal::init(Default::default());
    /// let mb = sifli_hal::mailbox::Mailbox2::new(p.MAILBOX2);
    ///
    /// let (ch1, ch2) = mb.split();
    ///
    /// spawner.spawn(control_task(ch1)).unwrap();
    /// spawner.spawn(data_task(ch2)).unwrap();
    /// # }
    /// ```
    pub fn split(
        self,
    ) -> (
        MailboxChannel<'d, peripherals::MAILBOX2, 0>,
        MailboxChannel<'d, peripherals::MAILBOX2, 1>,
    ) {
        (self.ch1, self.ch2)
    }
}
