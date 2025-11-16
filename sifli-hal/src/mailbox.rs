//! Mailbox HAL driver
//!
//! The Mailbox HAL driver provides high-level APIs for using the hardware mailbox module.
//! Each subsystem has a hardware mailbox module that can be used to:
//! - Trigger interrupts to notify other subsystems (e.g., HPSYS mailbox group H2L_MAILBOX
//!   triggers LPSYS interrupts)
//! - Protect shared hardware resources across multiple subsystems using mutex channels
//!
//! ## Features
//!
//! - Each mailbox group has 16 channels, allowing simultaneous triggering of all interrupts
//! - Mailbox interrupts can automatically wake up subsystems in LIGHT/DEEP/STANDBY low-power modes
//! - Mutex channels to protect shared resources, accessible from all subsystems
//!
//! ## Available Resources
//!
//! ### HPSYS (High-Power Subsystem)
//! - `MAILBOX1` (H2L_MAILBOX) - Mailbox for HCPU to LCPU communication
//! - Mutex channels (HMUTEX_CH1-4)
//!
//! ### LPSYS (Low-Power Subsystem)
//! - `MAILBOX2` (L2H_MAILBOX) - Mailbox for LCPU to HCPU communication
//! - Mutex channels (LMUTEX_CH1-2)
//!
//! ## Hardware Register Structure
//!
//! Each mailbox channel has the following registers:
//! - CxIER: Interrupt Enable Register
//! - CxITR: Interrupt Trigger Register
//! - CxICR: Interrupt Clear Register
//! - CxISR: Interrupt Status Register
//! - CxMISR: Masked Interrupt Status Register
//! - CxEXR: Exclusive (Mutex) Register
//!
//! ## Note
//!
//! This is a hardware abstraction layer that provides safe access to mailbox functionality.
//! The actual peripheral instances and interrupts must be properly configured in the PAC.
//! Currently, this module provides the types and interfaces, with actual peripheral
//! implementations to be added when PAC support is available.

use crate::_generated::interrupt::typelevel::Interrupt;

/// Mailbox channel identifier (0-15)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel(u8);

impl Channel {
    /// Maximum number of channels
    pub const MAX_CHANNELS: u8 = 16;

    /// Create a new channel (0-15)
    ///
    /// Returns `None` if the channel number is >= 16
    #[inline]
    pub const fn new(channel: u8) -> Option<Self> {
        if channel < Self::MAX_CHANNELS {
            Some(Channel(channel))
        } else {
            None
        }
    }

    /// Create a new channel without bounds checking
    ///
    /// # Safety
    /// The caller must ensure that `channel` is in range 0-15
    #[inline]
    pub const unsafe fn new_unchecked(channel: u8) -> Self {
        Channel(channel)
    }

    /// Convert channel to bit mask
    #[inline]
    pub const fn mask(self) -> u32 {
        1u32 << self.0
    }

    /// Get channel index
    #[inline]
    pub const fn index(self) -> u8 {
        self.0
    }
}

// Predefined channel constants for convenience
impl Channel {
    pub const CH0: Self = Channel(0);
    pub const CH1: Self = Channel(1);
    pub const CH2: Self = Channel(2);
    pub const CH3: Self = Channel(3);
    pub const CH4: Self = Channel(4);
    pub const CH5: Self = Channel(5);
    pub const CH6: Self = Channel(6);
    pub const CH7: Self = Channel(7);
    pub const CH8: Self = Channel(8);
    pub const CH9: Self = Channel(9);
    pub const CH10: Self = Channel(10);
    pub const CH11: Self = Channel(11);
    pub const CH12: Self = Channel(12);
    pub const CH13: Self = Channel(13);
    pub const CH14: Self = Channel(14);
    pub const CH15: Self = Channel(15);
}

/// Mailbox driver state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Mailbox not yet initialized or disabled
    Reset,
    /// Mailbox initialized and ready for use
    Ready,
    /// Mailbox internal processing is ongoing
    Busy,
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
    fn from_u8(value: u8) -> Self {
        match value {
            0 => LockCore::Unlocked,
            1 => LockCore::Hcpu,
            2 => LockCore::Lcpu,
            3 => LockCore::Bcpu,
            _ => LockCore::Unlocked,
        }
    }

    /// Check if the mutex is locked
    #[inline]
    pub const fn is_locked(self) -> bool {
        !matches!(self, LockCore::Unlocked)
    }
}

// /// Get a pointer to MAILBOX1 channel registers
// ///
// /// # Safety
// /// Caller must ensure exclusive access and proper synchronization
// pub unsafe fn mailbox1_regs() -> &'static mut MailboxChannelRegs {
//     &mut *(addrs::MAILBOX1_BASE as *mut MailboxChannelRegs)
// }

// /// Get a pointer to MAILBOX2 channel registers
// ///
// /// # Safety
// /// Caller must ensure exclusive access and proper synchronization
// pub unsafe fn mailbox2_regs() -> &'static mut MailboxChannelRegs {
//     &mut *(addrs::MAILBOX2_BASE as *mut MailboxChannelRegs)
// }

// TODO: When PAC support is available, implement the following:
// - Mailbox driver struct with Peripheral trait
// - Interrupt handlers
// - Integration with embassy_hal patterns
// - Proper RCC enable/reset support

use core::marker::PhantomData;

// Example of future API once PAC is available:
//
// ```rust,ignore
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use sifli_pac::mailbox::Mailbox1;

use crate::{interrupt, rcc::RccEnableReset};
//
pub struct Mailbox<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Mailbox<'d, T> {
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        into_ref!(peri);
        // T::enable_and_reset();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self { _peri: peri }
    }

    pub fn mask_channel(&mut self, channel: Channel) {
        T::regs().mask_channel(channel);
    }

    // ... other methods
}

/// Helper functions for mailbox operations
impl<'d, T: Instance> Mailbox<'d, T> {
    /// Mask (disable) interrupt for the specified channel
    #[inline]
    pub fn mask_channel(&mut self, channel: Channel) {
        let r = T::regs();
        todo!()
    }

    /// Unmask (enable) interrupt for the specified channel
    #[inline]
    pub fn unmask_channel(&mut self, channel: Channel) {
        let r = T::regs();
        todo!()
    }

    /// Trigger interrupt on the specified channel
    #[inline]
    pub fn trigger_channel(&mut self, channel: Channel) {
        let r = T::regs();
        todo!()
        // r.itr(channel.index().into())
        //     .write(|itr| itr.set_int(channel.index() as usize, true));
    }

    /// Check if interrupt is pending on the specified channel
    #[inline]
    pub fn is_channel_pending(&self, channel: Channel) -> bool {
        todo!()
    }

    /// Clear interrupt on the specified channel
    #[inline]
    pub fn clear_channel(&mut self, channel: Channel) {
        todo!()
    }

    /// Get the masked interrupt status (all channels)
    #[inline]
    pub fn get_status(&self) -> u32 {
        todo!()
    }
}

pub trait Instance: RccEnableReset + 'static {
    // type Interrupt: interrupt::typelevel::Interrupt;
    fn regs() -> Mailbox1;
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

// impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
//     unsafe fn on_interrupt() {
//         todo!()
//     }
// }

impl Instance for crate::peripherals::MAILBOX1 {
    // The interrupt triggers when hcpu receives data from lcpu from MAILBOX2?
    // type Interrupt = crate::interrupt::typelevel::MAILBOX2_CH1;

    fn regs() -> Mailbox1 {
        crate::pac::MAILBOX1
    }
}
