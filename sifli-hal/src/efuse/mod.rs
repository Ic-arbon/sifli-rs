//! EFUSE (eFuse controller)

use core::marker::PhantomData;

use embassy_hal_internal::Peripheral;

use crate::pac::EFUSEC;
use crate::{peripherals, rcc};

/// EFUSE error.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// PCLK frequency is unknown.
    PclkUnknown,
    /// PCLK is higher than the supported limit.
    PclkTooFast { pclk_hz: u32 },
    /// A timing value does not fit in the EFUSE timing register.
    TimingOutOfRange { field: &'static str, value: u32 },
}

/// EFUSE driver.
///
/// This is currently a minimal skeleton that only initializes controller timings.
pub struct Efuse<'d> {
    _phantom: PhantomData<&'d peripherals::EFUSEC>,
}

impl<'d> Efuse<'d> {
    /// Create a new EFUSE driver and initialize the controller timing register.
    pub fn new(_efusec: impl Peripheral<P = peripherals::EFUSEC> + 'd) -> Result<Self, Error> {
        rcc::enable_and_reset::<peripherals::EFUSEC>();
        init_timr()?;
        Ok(Self {
            _phantom: PhantomData,
        })
    }
}

fn init_timr() -> Result<(), Error> {
    let pclk_hz = rcc::get_pclk_freq().ok_or(Error::PclkUnknown)?.0;

    // CSDK: EFUSE_PCLK_LIMIT = 120000000
    #[cfg(not(feature = "unchecked-overclocking"))]
    if pclk_hz > 120_000_000 {
        return Err(Error::PclkTooFast { pclk_hz });
    }

    let (thrck, thpck, tckhp) = compute_timings(pclk_hz)?;
    EFUSEC.timr().write(|w| {
        w.set_thrck(thrck);
        w.set_thpck(thpck);
        w.set_tckhp(tckhp);
    });

    Ok(())
}

fn compute_timings(pclk_hz: u32) -> Result<(u8, u8, u16), Error> {
    // From CSDK `HAL_EFUSE_Init` (drivers/hal/bf0_hal_efuse.c).

    // EFUSE_RD_TIM_NS = 500
    let rd_thrck = (500u64 * pclk_hz as u64) / 1_000_000_000u64 + 1;
    if rd_thrck > 0x7f {
        return Err(Error::TimingOutOfRange {
            field: "thrck",
            value: rd_thrck as u32,
        });
    }

    // EFUSE_PGM_THPCK_NS = 20
    let pgm_thpck = (20u64 * pclk_hz as u64) / 1_000_000_000u64 + 1;
    if pgm_thpck > 0x07 {
        return Err(Error::TimingOutOfRange {
            field: "thpck",
            value: pgm_thpck as u32,
        });
    }

    // EFUSE_PGM_TCKHP_US = 10
    let mut pgm_tckhp = ((10u64 * pclk_hz as u64) + 500_000) / 1_000_000u64;
    let pgm_tckhp_ns = (pgm_tckhp * 1_000_000_000u64) / (pclk_hz as u64);
    if pgm_tckhp_ns > 11_000 {
        pgm_tckhp = pgm_tckhp.saturating_sub(1);
    } else if pgm_tckhp_ns < 9_000 {
        pgm_tckhp += 1;
    }
    if pgm_tckhp > 0x07ff {
        return Err(Error::TimingOutOfRange {
            field: "tckhp",
            value: pgm_tckhp as u32,
        });
    }

    Ok((rd_thrck as u8, pgm_thpck as u8, pgm_tckhp as u16))
}

