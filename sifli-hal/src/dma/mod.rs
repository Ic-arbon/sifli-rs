//! Direct Memory Access (DMA)

// The following code is modified from embassy-stm32 under MIT license
// https://github.com/embassy-rs/embassy/tree/main/embassy-stm32
// Special thanks to the Embassy Project and its contributors for their work!
#![macro_use]

mod dma;
use core::marker::PhantomData;

pub use dma::*;

mod util;
pub(crate) use util::*;

pub(crate) mod ringbuffer;
pub mod word;

use embassy_hal_internal::{impl_peripheral, Peripheral};

use crate::interrupt;
pub type Request = u8;

pub(crate) trait SealedChannel {
    fn id(&self) -> u8;
}

/// DMA channel.
#[allow(private_bounds)]
pub trait Channel: SealedChannel + Peripheral<P = Self> + Into<AnyChannel> + 'static {
    /// Type-erase (degrade) this pin into an `AnyChannel`.
    ///
    /// This converts DMA channel singletons (`DMA1_CH3`, `DMA2_CH1`, ...), which
    /// are all different types, into the same type. It is useful for
    /// creating arrays of channels, or avoiding generics.
    #[inline]
    fn degrade(self) -> AnyChannel {
        AnyChannel { id: self.id() }
    }

    type Interrupt: interrupt::typelevel::Interrupt;
}

pub struct InterruptHandler<T: Channel> {
    _phantom: PhantomData<T>,
}

impl<T: Channel> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        T::degrade(T).on_irq();
    }
}

/// Type-erased DMA channel.
pub struct AnyChannel {
    pub(crate) id: u8,
}
impl_peripheral!(AnyChannel);

impl AnyChannel {
    fn info(&self) -> &ChannelInfo {
        // This relies on generated code from build.rs
        // &crate::_generated::dmac::DMAC_CHANNELS[self.id as usize]
        todo!("Implement `info` method for AnyChannel");
    }
}

impl SealedChannel for AnyChannel {
    fn id(&self) -> u8 {
        self.id
    }
}
// TODO
// impl Channel for AnyChannel {}

const CHANNEL_COUNT: usize = 8; // Assuming 8 channels for DMAC1
static STATE: [dma::ChannelState; CHANNEL_COUNT] = [dma::ChannelState::NEW; CHANNEL_COUNT];

pub(crate) unsafe fn init(cs: critical_section::CriticalSection, dma_priority: interrupt::Priority) {
    dma::init(cs, dma_priority);
}
