//! Audio Processor driver (stub).
//!
//! Borrows `&AudioPll` to ensure the PLL outlives this driver.

use embassy_hal_internal::into_ref;

use crate::aud_pll::{AudioPll, SampleRate};
use crate::rcc;
use crate::{peripherals, Peripheral};

pub struct Config {
    pub sample_rate: SampleRate,
}

pub struct AudPrc<'d> {
    _peri: crate::PeripheralRef<'d, peripherals::AUDPRC>,
    _pll: &'d AudioPll,
}

impl<'d> AudPrc<'d> {
    /// # Panics
    ///
    /// Panics if `config.sample_rate` requires a different PLL frequency than `pll`.
    pub fn new(
        peri: impl Peripheral<P = peripherals::AUDPRC> + 'd,
        pll: &'d AudioPll,
        config: Config,
    ) -> Self {
        into_ref!(peri);
        pll.assert_compatible(config.sample_rate);
        rcc::enable_and_reset::<peripherals::AUDPRC>();
        todo!()
    }
}
