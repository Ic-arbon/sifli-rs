//! Audio PLL management (AUDCODEC PLL).
//!
//! The Audio PLL is a dedicated clock source for audio peripherals (I2S1, AUDPRC, PDM1).
//! It is **not** managed by `rcc::ConfigBuilder`/`init()`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use sifli_hal::aud_pll::{AudioPll, AudPllFreq, SampleRate};
//!
//! let pll = AudioPll::new(AudPllFreq::Mhz49_152);
//!
//! // Drivers borrow &pll — the PLL cannot be dropped while drivers exist.
//! let i2s = I2s::new(p.I2S1, &pll, i2s::Config { sample_rate: SampleRate::Hz48000 });
//! ```
//!
//! Reference: SDK `bf0_hal_audcodec_m.c` `bf0_enable_pll` / `HAL_TURN_OFF_PLL`.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::pac::{AUDCODEC, PMUC};
use crate::{cortex_m_blocking_delay_us, rcc};
use crate::time::Hertz;

/// Audio PLL output frequency.
///
/// Three SDK-validated frequencies derived from 48MHz XTAL:
/// `Fout = [(FCW+3) + SDIN/2^20] × 6MHz` (XTAL 48MHz / 8 = 6MHz Fref)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AudPllFreq {
    /// 49.152 MHz — 48k family (×1024)
    Mhz49_152,
    /// 45.1584 MHz — 44.1k family (×1024)
    Mhz45_1584,
    /// 44.1 MHz — 44.1k family (×1000)
    Mhz44_1,
}

impl AudPllFreq {
    /// PLL output frequency.
    pub const fn freq(&self) -> Hertz {
        Hertz(match self {
            Self::Mhz49_152 => 49_152_000,
            Self::Mhz45_1584 => 45_158_400,
            Self::Mhz44_1 => 44_100_000,
        })
    }

    pub(crate) const fn fcw(&self) -> u8 {
        match self {
            Self::Mhz49_152 => 5,
            Self::Mhz45_1584 => 4,
            Self::Mhz44_1 => 4,
        }
    }

    pub(crate) const fn sdin(&self) -> u32 {
        match self {
            Self::Mhz49_152 => 201327,
            Self::Mhz45_1584 => 551970,
            Self::Mhz44_1 => 366_874,
        }
    }
}

/// Audio sample rate.
///
/// Compile-time mapping from sample rate to required PLL frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SampleRate {
    Hz8000,
    Hz11025,
    Hz12000,
    Hz16000,
    Hz22050,
    Hz24000,
    Hz32000,
    Hz44100,
    Hz48000,
    Hz96000,
    Hz192000,
}

impl SampleRate {
    /// Sample rate frequency in Hz.
    pub const fn freq(&self) -> Hertz {
        Hertz(match self {
            Self::Hz8000 => 8_000,
            Self::Hz11025 => 11_025,
            Self::Hz12000 => 12_000,
            Self::Hz16000 => 16_000,
            Self::Hz22050 => 22_050,
            Self::Hz24000 => 24_000,
            Self::Hz32000 => 32_000,
            Self::Hz44100 => 44_100,
            Self::Hz48000 => 48_000,
            Self::Hz96000 => 96_000,
            Self::Hz192000 => 192_000,
        })
    }

    /// Required Audio PLL frequency for this sample rate.
    ///
    /// 48k family (8k/12k/16k/24k/32k/48k/96k/192k) → 49.152 MHz (×1024)
    /// 44.1k family (11.025k/22.05k/44.1k) → 45.1584 MHz (×1024)
    pub const fn pll_freq(&self) -> AudPllFreq {
        match self {
            Self::Hz8000
            | Self::Hz12000
            | Self::Hz16000
            | Self::Hz24000
            | Self::Hz32000
            | Self::Hz48000
            | Self::Hz96000
            | Self::Hz192000 => AudPllFreq::Mhz49_152,
            Self::Hz11025 | Self::Hz22050 | Self::Hz44100 => AudPllFreq::Mhz45_1584,
        }
    }
}

// =============================================================================
// AudioPll struct
// =============================================================================

/// # Safety Warning (runtime)
///
/// `AudioPll` uniqueness is checked at runtime via `AtomicBool`. Creating a
/// second instance will panic.
static TAKEN: AtomicBool = AtomicBool::new(false);

/// Audio PLL handle.
///
/// Owns the AUDCODEC PLL hardware. Drivers borrow `&AudioPll` to ensure the
/// PLL outlives them (compile-time lifetime guarantee).
///
/// Only one `AudioPll` can exist at a time (enforced at runtime via panic).
pub struct AudioPll {
    freq: AudPllFreq,
}

impl AudioPll {
    /// Create and enable the Audio PLL at the given frequency.
    ///
    /// # Panics
    ///
    /// Panics if an `AudioPll` already exists. Only one instance is allowed.
    pub fn new(freq: AudPllFreq) -> Self {
        assert!(!TAKEN.swap(true, Ordering::AcqRel), "AudioPll already created");

        // Step 1: Enable HXT audio buffer and AUDCODEC clock gate
        PMUC.hxt_cr1().modify(|w| w.set_buf_aud_en(true));
        rcc::enable::<crate::peripherals::AUDCODEC>();

        // Step 2: Bandgap
        AUDCODEC.bg_cfg0().modify(|w| w.set_en_rcflt(false));
        AUDCODEC.bg_cfg0().modify(|w| w.set_en(true));
        cortex_m_blocking_delay_us(100);
        AUDCODEC.bg_cfg0().modify(|w| w.set_en_smpl(false));
        cortex_m_blocking_delay_us(100);

        // Step 3: PLL analog
        AUDCODEC.pll_cfg0().modify(|w| w.set_en_iary(true));
        AUDCODEC.pll_cfg0().modify(|w| w.set_en_vco(true));
        AUDCODEC.pll_cfg0().modify(|w| w.set_en_ana(true));
        AUDCODEC.pll_cfg0().modify(|w| w.set_icp_sel(8));

        // PLL digital
        AUDCODEC.pll_cfg2().modify(|w| w.set_en_dig(true));
        AUDCODEC.pll_cfg3().modify(|w| w.set_en_sdm(true));
        AUDCODEC.pll_cfg4().modify(|w| w.set_en_clk_dig(true));

        // Loop filter: R3=3, RZ=1, C2=3, CZ=6, CSD off
        AUDCODEC.pll_cfg1().modify(|w| {
            w.set_r3_sel(3);
            w.set_rz_sel(1);
            w.set_c2_sel(3);
            w.set_cz_sel(6);
            w.set_csd_rst(false);
            w.set_csd_en(false);
        });
        cortex_m_blocking_delay_us(50);

        // Step 4: VCO calibration
        vco_calibrate();

        // Step 5: Set SDM frequency and verify lock
        set_sdm_freq(freq);

        // Update Clocks cache
        update_clocks_cache(freq);

        Self { freq }
    }

    /// Get the configured PLL frequency.
    pub fn freq(&self) -> AudPllFreq {
        self.freq
    }

    /// # Safety Warning (runtime)
    ///
    /// Sample rate family compatibility is a **runtime check**. The hardware has
    /// only one PLL and cannot output two frequencies simultaneously.
    /// 48k family requires `Mhz49_152`, 44.1k family requires `Mhz45_1584`.
    /// Mismatches will panic.
    pub(crate) fn assert_compatible(&self, sample_rate: SampleRate) {
        let required = sample_rate.pll_freq();
        assert!(
            self.freq == required,
            "SampleRate {:?} requires {:?}, but AudioPll is configured for {:?}",
            sample_rate,
            required,
            self.freq,
        );
    }
}

impl Drop for AudioPll {
    fn drop(&mut self) {
        // PLL off
        AUDCODEC.pll_cfg0().modify(|w| {
            w.set_en_iary(false);
            w.set_en_vco(false);
            w.set_en_ana(false);
        });
        AUDCODEC.pll_cfg2().modify(|w| w.set_en_dig(false));
        AUDCODEC.pll_cfg3().modify(|w| w.set_en_sdm(false));
        AUDCODEC.pll_cfg4().modify(|w| w.set_en_clk_dig(false));

        // RefGen off
        AUDCODEC.refgen_cfg().modify(|w| w.set_en(false));

        // Bandgap off
        AUDCODEC.bg_cfg1().write(|w| w.0 = 0);
        AUDCODEC.bg_cfg2().write(|w| w.0 = 0);
        AUDCODEC.bg_cfg0().modify(|w| {
            w.set_en(false);
            w.set_en_smpl(false);
        });

        // Update Clocks cache
        unsafe {
            let mut clocks = *crate::rcc::get_freqs();
            clocks.clk_aud_pll = None.into();
            clocks.clk_aud_pll_div16 = None.into();
            crate::rcc::set_freqs(clocks);
        }

        TAKEN.store(false, Ordering::Release);
    }
}

/// Check if the Audio PLL is currently enabled (reads hardware).
pub fn is_enabled() -> bool {
    AUDCODEC.pll_cfg0().read().en_ana()
}

// =============================================================================
// Internal helpers
// =============================================================================

/// VCO open-loop calibration: binary search + neighbor refinement.
///
/// Target: fc_vco that gives pll_cnt closest to 1838 (≈44MHz VCO free-run)
/// at calibration length 2000.
fn vco_calibrate() {
    const TARGET_CNT: u16 = 1838;
    const CAL_LEN: u16 = 2000;

    // Enter open-loop mode
    AUDCODEC.pll_cfg0().modify(|w| w.set_open(true));
    AUDCODEC.pll_cfg2().modify(|w| w.set_en_lf_vcin(true));
    AUDCODEC.pll_cal_cfg().write(|w| {
        w.set_en(false);
        w.set_len(CAL_LEN);
    });

    // Phase 1: Binary search
    let mut fc_vco: u8 = 16;
    let mut delta: u8 = 8;
    let mut best_cnt: u16 = 0;

    while delta != 0 {
        let cnt = measure_vco(fc_vco);
        best_cnt = cnt;

        if cnt < TARGET_CNT {
            fc_vco = fc_vco.saturating_add(delta);
        } else if cnt > TARGET_CNT {
            fc_vco = fc_vco.saturating_sub(delta);
        }
        delta >>= 1;
    }
    // Re-measure to ensure best_cnt corresponds to current fc_vco
    best_cnt = measure_vco(fc_vco);

    // Phase 2: Neighbor refinement — test fc_vco-1, fc_vco, fc_vco+1
    let fc_min = fc_vco.saturating_sub(1);
    let fc_max = if fc_vco < 31 { fc_vco + 1 } else { fc_vco };

    let cnt_min = measure_vco(fc_min);
    let cnt_max = measure_vco(fc_max);

    let delta_mid = (best_cnt as i32 - TARGET_CNT as i32).unsigned_abs();
    let delta_lo = (cnt_min as i32 - TARGET_CNT as i32).unsigned_abs();
    let delta_hi = (cnt_max as i32 - TARGET_CNT as i32).unsigned_abs();

    let best_fc = if delta_lo <= delta_mid && delta_lo <= delta_hi {
        fc_min
    } else if delta_hi <= delta_mid && delta_hi <= delta_lo {
        fc_max
    } else {
        fc_vco
    };

    AUDCODEC.pll_cfg0().modify(|w| w.set_fc_vco(best_fc));

    // Exit open-loop mode
    AUDCODEC.pll_cfg2().modify(|w| w.set_en_lf_vcin(false));
    AUDCODEC.pll_cfg0().modify(|w| w.set_open(false));
    cortex_m_blocking_delay_us(50);
}

/// Run one VCO calibration measurement at the given fc_vco value.
fn measure_vco(fc_vco: u8) -> u16 {
    AUDCODEC.pll_cfg0().modify(|w| w.set_fc_vco(fc_vco));

    AUDCODEC.pll_cal_cfg().modify(|w| w.set_en(true));
    while !AUDCODEC.pll_cal_cfg().read().done() {}

    let result = AUDCODEC.pll_cal_result().read();
    let pll_cnt = result.pll_cnt();

    AUDCODEC.pll_cal_cfg().modify(|w| w.set_en(false));

    pll_cnt
}

/// Set the SDM frequency parameters and verify CSD lock.
fn set_sdm_freq(freq: AudPllFreq) {
    // Release reset
    AUDCODEC.pll_cfg2().modify(|w| w.set_rstb(true));
    cortex_m_blocking_delay_us(50);

    // Write FCW + SDIN + SDM control
    AUDCODEC.pll_cfg3().write(|w| {
        w.set_sdin(freq.sdin());
        w.set_fcw(freq.fcw());
        w.set_sdm_update(false);
        w.set_sdmin_bypass(true);
        w.set_sdm_mode(false);
        w.set_en_sdm_dither(false);
        w.set_sdm_dither(false);
        w.set_en_sdm(true);
        w.set_sdmclk_pol(false);
    });

    // SDM update sequence
    AUDCODEC.pll_cfg3().modify(|w| w.set_sdm_update(true));
    AUDCODEC.pll_cfg3().modify(|w| w.set_sdmin_bypass(false));

    // Reset cycle for SDM
    AUDCODEC.pll_cfg2().modify(|w| w.set_rstb(false));
    cortex_m_blocking_delay_us(50);
    AUDCODEC.pll_cfg2().modify(|w| w.set_rstb(true));
    cortex_m_blocking_delay_us(50);

    // CSD lock detection
    AUDCODEC.pll_cfg1().modify(|w| {
        w.set_csd_en(true);
        w.set_csd_rst(true);
    });
    cortex_m_blocking_delay_us(50);
    AUDCODEC.pll_cfg1().modify(|w| w.set_csd_rst(false));

    if AUDCODEC.pll_stat().read().unlock() {
        warn!("Audio PLL failed to lock");
    }

    AUDCODEC.pll_cfg1().modify(|w| w.set_csd_en(false));
}

fn update_clocks_cache(freq: AudPllFreq) {
    let pll_hz = freq.freq();
    unsafe {
        let mut clocks = *crate::rcc::get_freqs();
        clocks.clk_aud_pll = Some(pll_hz).into();
        clocks.clk_aud_pll_div16 = Some(Hertz(pll_hz.0 / 16)).into();
        crate::rcc::set_freqs(clocks);
    }
}
