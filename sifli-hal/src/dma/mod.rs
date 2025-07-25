//! Direct Memory Access (DMA)

// The following code is modified from embassy-stm32 under MIT license
// https://github.com/embassy-rs/embassy/tree/main/embassy-stm32
// Special thanks to the Embassy Project and its contributors for their work!
#![macro_use]

mod dma;
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

pub(crate) trait ChannelInterrupt {
    #[cfg_attr(not(feature = "rt"), allow(unused))]
    unsafe fn on_irq();
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
}

macro_rules! dma_channel_impl {
    ($channel_peri:ident, $index:expr) => {
        impl crate::dma::SealedChannel for crate::peripherals::$channel_peri {
            fn id(&self) -> u8 {
                $index
            }
        }
        impl crate::dma::ChannelInterrupt for crate::peripherals::$channel_peri {
            unsafe fn on_irq() {
                crate::dma::AnyChannel { id: $index }.on_irq();
            }
        }

        impl crate::dma::Channel for crate::peripherals::$channel_peri {}

        impl From<crate::peripherals::$channel_peri> for crate::dma::AnyChannel {
            fn from(val: crate::peripherals::$channel_peri) -> Self {
                Self {
                    id: crate::dma::SealedChannel::id(&val),
                }
            }
        }
    };
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
impl Channel for AnyChannel {}

const CHANNEL_COUNT: usize = 8; // Assuming 8 channels for DMAC1
static STATE: [dma::ChannelState; CHANNEL_COUNT] = [dma::ChannelState::NEW; CHANNEL_COUNT];

pub(crate) unsafe fn init(cs: critical_section::CriticalSection, dma_priority: interrupt::Priority) {
    dma::init(cs, dma_priority);
}

dma_channel_impl!(DMAC_CH1, 0);
dma_channel_impl!(DMAC_CH2, 1);
dma_channel_impl!(DMAC_CH3, 2);
dma_channel_impl!(DMAC_CH4, 3);
dma_channel_impl!(DMAC_CH5, 4);
dma_channel_impl!(DMAC_CH6, 5);
dma_channel_impl!(DMAC_CH7, 6);
dma_channel_impl!(DMAC_CH8, 7);