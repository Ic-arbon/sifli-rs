//! LCPU-side Bluetooth RF calibration implementation.
//!
//! This module implements the complete RF calibration algorithms from SDK `bt_rf_fulcal.c`,
//! including:
//! - RFC initialization (command sequence generation)
//! - VCO calibration (ACAL/FCAL)
//! - EDR LO calibration (3GHz mode)
//! - TXDC calibration
//! - Optimization calibration (PA/PHY configuration)
//!
//! The calibration process is typically run during LCPU startup after patch installation.

pub mod regs;
pub mod tables;

mod rfc_init;
pub use rfc_init::rfc_init;

mod ful_cal;
pub use ful_cal::ful_cal;

mod opt_cal;
pub use opt_cal::opt_cal;

/// EDR mode selection for RF calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EdrMode {
    /// 3GHz VCO for IQ TX (default, most common)
    #[default]
    Edr3G,
    /// 5GHz VCO for IQ TX
    Edr5G,
    /// 2GHz VCO for IQ TX
    Edr2G,
}

/// RF calibration configuration.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RfCalConfig {
    /// Maximum TX power in dBm (default 10).
    pub max_tx_power: i8,
    /// Minimum TX power in dBm (default 0).
    pub min_tx_power: i8,
    /// Initial TX power in dBm (default 0).
    pub init_tx_power: i8,
    /// BQB (Bluetooth Qualification Body) test mode.
    pub is_bqb_mode: bool,
    /// EDR VCO mode selection.
    pub edr_mode: EdrMode,
    /// Enable full RF calibration (VCO/TXDC/IQ).
    /// If false, only basic reset and power configuration is performed.
    pub enable_full_cal: bool,
}

impl Default for RfCalConfig {
    fn default() -> Self {
        Self {
            max_tx_power: 10,
            min_tx_power: 0,
            init_tx_power: 0,
            is_bqb_mode: false,
            edr_mode: EdrMode::default(),
            enable_full_cal: true,
        }
    }
}

impl RfCalConfig {
    /// Create a minimal configuration (no full calibration).
    pub const fn minimal() -> Self {
        Self {
            max_tx_power: 10,
            min_tx_power: 0,
            init_tx_power: 0,
            is_bqb_mode: false,
            edr_mode: EdrMode::Edr3G,
            enable_full_cal: false,
        }
    }

    /// Calculate power calibration enable mask based on power range.
    ///
    /// Returns a bitmask indicating which power levels (0-6) should be calibrated.
    /// Power levels correspond to: 0dBm, 3dBm, 6dBm, 10dBm, 13dBm, 16dBm, 19dBm.
    pub fn cal_power_mask(&self) -> u8 {
        const PWR_TABLE: [i8; 7] = [0, 3, 6, 10, 13, 16, 19];

        let min_pwr = self.min_tx_power;
        let max_pwr = self.max_tx_power.max(self.init_tx_power);

        let mut min_level = 0u8;
        let mut max_level = 6u8;

        // Find minimum level
        for i in (0..7).rev() {
            if PWR_TABLE[i] <= min_pwr {
                min_level = i as u8;
                break;
            }
        }

        // Find maximum level
        for i in 0..7 {
            if PWR_TABLE[i] >= max_pwr {
                max_level = i as u8;
                break;
            }
        }

        // Build enable mask
        let mut mask = 0u8;
        for i in min_level..=max_level {
            mask |= 1 << i;
        }
        mask
    }
}
