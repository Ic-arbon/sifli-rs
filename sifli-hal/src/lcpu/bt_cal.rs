//! LCPU-side Bluetooth RF calibration.
//!
//! This module implements the complete RF calibration flow from SDK `bt_rf_cal()`:
//! - Resetting Bluetooth RF module via LPSYS_RCC
//! - RFC initialization and command sequence generation
//! - VCO calibration (ACAL/FCAL for 5G and 3G)
//! - TXDC calibration
//! - Optimization calibration (PA/PHY configuration)
//! - Writing BT transmit power parameters to LCPU ROM configuration area
//!
//! Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/bt_rf_fulcal.c`

use super::bt_rf;
use super::ram;
use crate::rcc::{lp_rfc_reset_asserted, set_lp_rfc_reset};
use crate::syscfg::ChipRevision;

#[allow(unused_imports)]
pub use bt_rf::{EdrMode, RfCalConfig};

/// Reset Bluetooth RF module.
///
/// Corresponds to `HAL_RCC_ResetBluetoothRF` in SDK.
fn reset_bluetooth_rf() {
    // Set RFC reset bit
    set_lp_rfc_reset(true);
    // Wait for bit to take effect
    while !lp_rfc_reset_asserted() {}
    // Clear RFC reset bit
    set_lp_rfc_reset(false);
}

/// Encode power parameters into 32-bit packed format.
///
/// Equivalent to `RF_PWR_PARA` macro in SDK:
/// `(is_bqb << 24) | (init << 16) | (min << 8) | (int8_t)(max)`.
fn encode_tx_power(max: i8, min: i8, init: i8, is_bqb: bool) -> u32 {
    let max_u = max as u8 as u32;
    let min_u = min as u8 as u32;
    let init_u = init as u8 as u32;
    let is_bqb_u = if is_bqb { 1u32 } else { 0u32 };

    (is_bqb_u << 24) | (init_u << 16) | (min_u << 8) | max_u
}

/// Perform Bluetooth RF calibration with default configuration.
///
/// This is a convenience wrapper for `bt_rf_cal_with_config` using default settings.
/// The default configuration performs full calibration with:
/// - Max TX power: 10 dBm
/// - Min TX power: 0 dBm
/// - Init TX power: 0 dBm
/// - EDR mode: 3G
/// - Full calibration enabled
#[allow(dead_code)]
pub fn bt_rf_cal(revision: ChipRevision) {
    bt_rf_cal_with_config(revision, &RfCalConfig::default());
}

/// Perform Bluetooth RF calibration with minimal configuration.
///
/// This performs only the essential steps (reset + power configuration) without
/// the complete VCO/TXDC calibration. Use this for faster startup when full
/// calibration is not required.
pub fn bt_rf_cal_minimal(revision: ChipRevision) {
    bt_rf_cal_with_config(revision, &RfCalConfig::minimal());
}

/// Perform Bluetooth RF calibration with custom configuration.
///
/// # Steps
///
/// 1. Reset Bluetooth RF module
/// 2. Initialize RFC (if full calibration enabled)
/// 3. Perform full calibration: VCO ACAL/FCAL, EDR LO, TXDC (if enabled)
/// 4. Perform optimization calibration (if full calibration enabled)
/// 5. Write TX power configuration to LCPU ROM
///
/// # Arguments
///
/// * `revision` - Chip revision for ROM configuration address selection
/// * `config` - RF calibration configuration
pub fn bt_rf_cal_with_config(revision: ChipRevision, config: &RfCalConfig) {
    // 1. Reset Bluetooth RF module
    reset_bluetooth_rf();

    // 2-4. Perform full calibration if enabled
    if config.enable_full_cal {
        // Initialize RFC and get calibration data start address
        let addr = bt_rf::rfc_init(config.edr_mode);

        // Perform full calibration (VCO/TXDC)
        let cal_power_mask = config.cal_power_mask();
        bt_rf::ful_cal(addr, config.edr_mode, cal_power_mask);

        // Perform optimization calibration
        bt_rf::opt_cal(config.edr_mode);
    }

    // 5. Write TX power configuration to LCPU ROM
    let tx_pwr = encode_tx_power(
        config.max_tx_power,
        config.min_tx_power,
        config.init_tx_power,
        config.is_bqb_mode,
    );
    ram::set_bt_tx_power(revision, tx_pwr);
}
