//! Optimization calibration implementation.
//!
//! This module implements `bt_rf_opt_cal()` from SDK, which configures
//! various RF parameters after the main calibration is complete:
//! - PA configuration
//! - RF baseband filter settings
//! - PHY demodulator configuration
//! - IQ modulation gain
//! - And more...

use super::regs::*;
use super::EdrMode;

// Additional PHY register offsets used by opt_cal
const MIXER_CFG1: usize = 0x48;
const DEMOD_CFG1: usize = 0x54;
const DEMOD_CFG8: usize = 0x70;
const DEMOD_CFG16: usize = 0x90;
const TED_CFG1: usize = 0x44;
const LFP_MMDIV_CFG0: usize = 0x130;
const LFP_MMDIV_CFG1: usize = 0x134;
const PKTDET_CFG2: usize = 0x50;
const TX_GAUSSFLT_CFG1: usize = 0xFC;
const TX_GAUSSFLT_CFG2: usize = 0x100;
const NOTCH_CFG1: usize = 0x08;
const NOTCH_CFG6: usize = 0x1C;
const NOTCH_CFG7: usize = 0x20;
const NOTCH_CFG8: usize = 0x24;
const NOTCH_CFG9: usize = 0x28;
const NOTCH_CFG10: usize = 0x2C;
const NOTCH_CFG11: usize = 0x30;
const INTERP_CFG1: usize = 0x40;
const EDRSYNC_CFG1: usize = 0xD8;
const EDRDEMOD_CFG1: usize = 0xE0;
const EDRDEMOD_CFG2: usize = 0xE4;
const EDRTED_CFG1: usize = 0xE8;
const LPF_CFG0: usize = 0x1CC;
const LPF_CFG1: usize = 0x1D0;
const LPF_CFG2: usize = 0x1D4;
const LPF_CFG3: usize = 0x1D8;

// Additional RFC register offsets
const RBB_REG1_OFF: usize = RBB_REG1;
const RBB_REG2_OFF: usize = RBB_REG2;
const RBB_REG4_OFF: usize = RBB_REG4;
const RBB_REG6_OFF: usize = RBB_REG6;

/// Perform optimization calibration.
///
/// This configures various RF parameters for optimal performance.
/// Must be called after the full calibration is complete.
pub fn opt_cal(edr_mode: EdrMode) {
    // PA configuration
    rfc_clear(TRF_REG1, BRF_PA_PM_LV_MSK | BRF_PA_CAS_BP_LV_MSK);
    rfc_clear(TRF_REG2, BRF_PA_UNIT_SEL_LV_MSK | BRF_PA_MCAP_LV_MSK);

    rfc_set(TRF_REG1, (0x01 << BRF_PA_PM_LV_POS) | (0x01 << BRF_PA_CAS_BP_LV_POS));
    rfc_set(TRF_REG2, (0x01 << BRF_PA_UNIT_SEL_LV_POS) | (0x0 << BRF_PA_MCAP_LV_POS));

    // RF CBPF - Peak detector thresholds
    rfc_modify(RBB_REG1_OFF, |v| {
        (v & 0xFFFF_0000)
            | (0x08 << 0)  // PKDET_VTH1I_BT
            | (0x08 << 4)  // PKDET_VTH1Q_BT
            | (0x08 << 8)  // PKDET_VTH2I_BT
            | (0x08 << 12) // PKDET_VTH2Q_BT
    });

    // CBPF_FC
    rfc_modify(RBB_REG2_OFF, |v| (v & !(0x7 << 0)) | (0x3 << 0));

    // Peak detector thresholds (LV)
    rfc_modify(RBB_REG4_OFF, |v| {
        (v & 0xFFFF_0000)
            | (0x0A << 0)  // PKDET_VTH1I_LV
            | (0x0A << 4)  // PKDET_VTH1Q_LV
            | (0x0A << 8)  // PKDET_VTH2I_LV
            | (0x0A << 12) // PKDET_VTH2Q_LV
    });

    // CBPF_BW_LV_BR
    rfc_modify(RBB_REG6_OFF, |v| {
        (v & !(0x7 << 0 | 0x1 << 3 | 0x1 << 4))
            | (0x1 << 0)  // CBPF_BW_LV_BR
            | (0x1 << 3)  // CBPF_W2X_STG1_LV_BR
            | (0x1 << 4)  // CBPF_W2X_STG2_LV_BR
    });

    // Enable ADC Q for all PHY modes
    phy_set(RX_CTRL1, ADC_Q_EN_1);
    phy_set(RX_CTRL2, ADC_Q_EN_FRC_EN);

    // Zero IF
    phy_clear(TX_IF_MOD_CFG, TX_IF_PHASE_BLE_MSK);

    // Release ADC reset
    rfc_set(ADC_REG, BRF_RSTB_ADC_LV);

    // Disable pkdet det early off
    rfc_clear(MISC_CTRL_REG, PKDET_EN_EARLY_OFF_EN);

    // Select VCO for IQ TX based on EDR mode
    match edr_mode {
        EdrMode::Edr5G => {
            phy_clear(TX_CTRL, MMDIV_SEL_MSK);
            rfc_modify(EDR_PLL_REG4, |v| (v & !(0x3 << 0)) | (1 << 0));
            rfc_modify(EDR_OSLO_REG, |v| (v & !(0x3 << 0)) | (1 << 0));
        }
        EdrMode::Edr3G => {
            phy_modify(TX_CTRL, |v| (v & !MMDIV_SEL_MSK) | (0x1 << MMDIV_SEL_POS));
        }
        EdrMode::Edr2G => {
            phy_modify(TX_CTRL, |v| (v & !MMDIV_SEL_MSK) | (0x2 << MMDIV_SEL_POS));
        }
    }

    // IQ modulation gain
    phy_write(TX_IF_MOD_CFG3, 0x8055_5555);
    phy_write(TX_IF_MOD_CFG5, 0x6855_5555);
    phy_write(TX_IF_MOD_CFG6, 0x4444_4444);
    phy_write(TX_IF_MOD_CFG7, 0x5050_5044);
    phy_write(TX_DPSK_CFG1, 0x4444_4444);
    phy_write(TX_DPSK_CFG2, 0x5050_5044);

    // Mixer configuration
    phy_modify(MIXER_CFG1, |v| {
        (v & !(0xFF << 0 | 0xFF << 8)) | (0xA6 << 0) | (0x80 << 8)
    });

    // MMDIV offset
    phy_modify(LFP_MMDIV_CFG0, |v| (v & !(0x1FFFF << 0)) | (0x1AAE1 << 0));
    phy_modify(LFP_MMDIV_CFG1, |v| (v & !(0x1FFFF << 0)) | (0x18000 << 0));

    // BLE demodulator configuration
    phy_modify(DEMOD_CFG1, |v| {
        (v & !(0xFF << 0 | 0xFF << 8 | 0x3FF << 16))
            | (0xB0 << 0)   // BLE_DEMOD_G
            | (0x22 << 8)   // BLE_MU_DC
            | (0x168 << 16) // BLE_MU_ERR
    });

    // BR demodulator configuration
    phy_modify(DEMOD_CFG8, |v| {
        (v & !(0xFF << 0 | 0xFF << 8 | 0x3FF << 16))
            | (0x50 << 0)   // BR_DEMOD_G
            | (0x10 << 8)   // BR_MU_DC
            | (0x120 << 16) // BR_MU_ERR
    });

    // Enable BR_HADAPT
    phy_set(DEMOD_CFG16, 1 << 0);

    // TED (Timing Error Detector) configuration
    phy_write(TED_CFG1, (0x02 << 0) | (0x04 << 4) | (0x03 << 8) | (0x05 << 12));

    // PKT detect threshold
    phy_modify(PKTDET_CFG2, |v| (v & !(0xFFF << 16)) | (0x500 << 16));

    // TX GFSK modulation index
    phy_modify(TX_GAUSSFLT_CFG1, |v| {
        (v & !(0xFF << 0 | 0xFF << 8 | 0xFF << 16))
            | (0xF7 << 0)  // POLAR_GAUSS_GAIN_2
            | (0xFD << 8)  // POLAR_GAUSS_GAIN_1
            | (0xAA << 16) // POLAR_GAUSS_GAIN_BR
    });

    phy_modify(TX_GAUSSFLT_CFG2, |v| {
        (v & !(0xFF << 0 | 0xFF << 8 | 0xFF << 16))
            | (0xAE << 0)  // IQ_GAUSS_GAIN_BR
            | (0xFF << 8)  // IQ_GAUSS_GAIN_1
            | (0xFF << 16) // IQ_GAUSS_GAIN_2
    });

    // NOTCH filter configuration
    phy_modify(NOTCH_CFG1, |v| (v & !(0xFFFF << 0)) | (0x3000 << 0));
    phy_modify(NOTCH_CFG7, |v| (v & !(0xFFFF_FFFF)) | (0x0000_4000));
    phy_modify(NOTCH_CFG10, |v| (v & !(0xFFFF_FFFF)) | (0x0000_4000));

    // Interpolator configuration
    phy_modify(INTERP_CFG1, |v| (v & !(0x3 << 0)) | (0x01 << 0));

    // EDR sync configuration
    phy_modify(EDRSYNC_CFG1, |v| (v & !(0x3 << 0)) | (0x01 << 0));

    // EDR demodulator configuration
    phy_modify(EDRDEMOD_CFG1, |v| {
        (v & !(0xFF << 0 | 0x3FF << 8)) | (0x40 << 0) | (0x100 << 8)
    });

    phy_modify(EDRDEMOD_CFG2, |v| {
        (v & !(0xFF << 0 | 0x3FF << 8)) | (0x40 << 0) | (0x140 << 8)
    });

    // BR_MU_H
    phy_modify(DEMOD_CFG16, |v| (v & !(0xFF << 8)) | (0x28 << 8));

    // EDR TED configuration
    phy_modify(EDRTED_CFG1, |v| {
        (v & !(0xF << 0 | 0xF << 4 | 0xF << 8 | 0xF << 12))
            | (0x8 << 0)  // TED_EDR2_MU_F
            | (0x4 << 4)  // TED_EDR2_MU_P
            | (0x8 << 8)  // TED_EDR3_MU_F
            | (0x4 << 12) // TED_EDR3_MU_P
    });

    // BT operation mode
    phy_set(RX_CTRL1, BT_OP_MODE);

    // LPF coefficients
    phy_modify(LPF_CFG0, |v| {
        (v & !(0x1FF << 0 | 0x1FF << 9 | 0x1FF << 18))
            | (0x8 << 0)    // LPF_COEF_0
            | (0x2 << 9)    // LPF_COEF_1
            | (0x1F1 << 18) // LPF_COEF_2
    });

    phy_modify(LPF_CFG1, |v| {
        (v & !(0x1FF << 0 | 0x1FF << 9 | 0x1FF << 18))
            | (0x1DF << 0)  // LPF_COEF_3
            | (0x1DD << 9)  // LPF_COEF_4
            | (0x1FE << 18) // LPF_COEF_5
    });

    phy_modify(LPF_CFG2, |v| {
        (v & !(0x1FF << 0 | 0x1FF << 9 | 0x1FF << 18))
            | (0x43 << 0)  // LPF_COEF_6
            | (0x9B << 9)  // LPF_COEF_7
            | (0xE5 << 18) // LPF_COEF_8
    });

    phy_modify(LPF_CFG3, |v| (v & !(0x1FF << 0)) | (0xFF << 0));

    // Additional NOTCH filter settings
    phy_write(NOTCH_CFG6, 0x0040_0000);
    phy_modify(NOTCH_CFG8, |v| (v & !(0xFF << 0)) | (0x40 << 0));
    phy_write(NOTCH_CFG9, 0x0040_0000);
    phy_modify(NOTCH_CFG11, |v| (v & !(0xFF << 0)) | (0x40 << 0));

    // Store driver version in reserved register
    rfc_write(RSVD_REG2, 0x0006_0000);
}
