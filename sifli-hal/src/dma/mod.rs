//! Direct Memory Access (DMA)

// The following code is modified from embassy-stm32 under MIT license
// https://github.com/embassy-rs/embassy/tree/main/embassy-stm32
// Special thanks to the Embassy Project and its contributors for their work!
#![macro_use]

use embassy_hal_internal::{impl_peripheral, Peripheral};

mod dma;
pub use dma::*;

mod util;
pub(crate) use util::*;

pub(crate) mod ringbuffer;
pub mod word;

pub use crate::_generated::Request;
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

/// Type-erased DMA channel.
pub struct AnyChannel {
    pub(crate) id: u8,
}
impl_peripheral!(AnyChannel);

// TODO: Multi DMAC in future?
impl AnyChannel {
    fn info(&self) -> ChannelInfo {
        ChannelInfo { dma: crate::pac::DMAC1, num: self.id as _ }
    }
}

impl SealedChannel for AnyChannel {
    fn id(&self) -> u8 {
        self.id
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
            fn from(x: crate::peripherals::$channel_peri) -> Self {
                crate::dma::Channel::degrade(x)
            }
        }
    };
}

impl Channel for AnyChannel {}

use crate::_generated::CHANNEL_COUNT;
static STATE: [dma::ChannelState; CHANNEL_COUNT] = [dma::ChannelState::NEW; CHANNEL_COUNT];

pub(crate) unsafe fn init(cs: critical_section::CriticalSection) {
    dma::init(cs);
}

/// "No DMA" placeholder.
///
/// You may pass this in place of a real DMA channel when creating a driver
/// to indicate it should not use DMA.
///
/// This often causes async functionality to not be available on the instance,
/// leaving only blocking functionality.
pub struct NoDma;

impl_peripheral!(NoDma);


// codegen will generate the following implementations
// We use this instead of `InterruptHandler` for the avaliblity of `AnyChannel`.

// ```
// dma_channel_impl!(DMAC1_CH1, 0);
// #[cfg(feature = "rt")]
// #[crate::interrupt]
// unsafe fn DMAC1_CH1() {
//     <crate::peripherals::DMAC1_CH1 as crate::dma::ChannelInterrupt>::on_irq();
// }
// ```
