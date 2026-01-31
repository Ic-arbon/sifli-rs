//! RFC initialization and command sequence generation.
//!
//! This module implements `bt_rfc_init()` from SDK, which:
//! - Enables ADC Q channel for all PHY modes
//! - Configures zero IF
//! - Resets RCCAL
//! - Selects EDR VCO mode
//! - Generates RXON/RXOFF/TXON/TXOFF command sequences

use super::regs::*;
use super::EdrMode;

/// RXON command sequence (BLE RX enable).
static RXON_CMD: [u16; 70] = [
    // VDDPSW/RFBG_EN/LO_IARY_EN
    rd(0x10),
    rd(0x10),
    or(18),
    or(17),
    or(16),
    wr(0x10), // 5
    // WAIT 1US
    wait(2),
    // FULCAL RSLT
    RD_FULCAL,
    wr(0x08),
    // VCO5G_EN
    rd(0x0),
    or(12),
    wr(0x0), // 11
    // PFDCP_EN
    rd(0x1C),
    or(19),
    wr(0x1C),
    // FBDV_EN
    rd(0x14),
    or(12),
    wr(0x14), // 17
    // FBDV_RSTB
    rd(0x14),
    and(7),
    wr(0x14),
    // wait 30us for lo lock
    wait(45), // 21
    // VCO_FLT_EN
    rd(0x0),
    or(7),
    wr(0x0),
    // LDO11_EN & LNA_SHUNTSW
    rd(0x44),
    or(22),
    and(6),
    wr(0x44), // 28
    // ADC & LDO_ADC & LDO_ADCREF (OR(20): if disable adc-1, change to 22)
    rd(0x60),
    or(4),
    or(9),
    or(21),
    or(20),
    wr(0x60),
    // LDO_RBB
    rd(0x48),
    or(13),
    wr(0x48), // 37
    // PA_TX_RX
    rd(0x38),
    and(9),
    wr(0x38),
    // EN_IARRAY & EN_OSDAC
    rd(0x58),
    or(5),
    or(6),
    or(7),
    wr(0x58), // 45
    // EN_CBPF & EN_RVGA
    rd(0x4C),
    or(27),
    or(6),
    or(7),
    wr(0x4C),
    // EN_PKDET
    rd(0x50),
    or(0),
    or(1),
    or(2),
    or(3),
    wr(0x50), // 56
    // wait 4us
    wait(5),
    // LODIST5G_RX_EN
    rd(0x10),
    or(9),
    wr(0x10), // 60
    // LNA_PU & MX_PU
    rd(0x44),
    or(3),
    or(17),
    wr(0x44),
    // START INCCAL, 0x74: inccal start
    rd(0x74),
    or(29),
    wr(0x74),
    wait(30), // 68
    // END should be put in odd
    END, // 69
];

/// RXOFF command sequence (BLE RX disable).
static RXOFF_CMD: [u16; 54] = [
    // VDDPSW/RFBG/LODIST5G_RX_EN/LO_IARY_EN
    rd(0x10),
    rd(0x10),
    and(18),
    and(17),
    and(16),
    and(9),
    wr(0x10), // 6
    // VCO5G_EN & VCO_FLT_EN
    rd(0x0),
    and(12),
    and(7),
    wr(0x00),
    // FBDV_EN / FBDV RSTB
    rd(0x14),
    and(12),
    or(7),
    wr(0x14), // 14
    // PFDCP_EN
    rd(0x1C),
    and(19),
    wr(0x1C),
    // LNA_PU & MX_PU & LDO11_EN & LNA_SHUNTSW
    rd(0x44),
    and(3),
    or(6),
    and(17),
    and(22),
    wr(0x44), // 23
    // ADC & LDO_ADC & LDO_ADCREF
    rd(0x60),
    and(4),
    and(9),
    and(21),
    and(20),
    wr(0x60),
    // LDO_RBB
    rd(0x48),
    and(13),
    wr(0x48), // 32
    // PA_TX_RX
    rd(0x38),
    or(9),
    wr(0x38),
    // EN_IARRAY & EN_OSDAC
    rd(0x58),
    and(5),
    and(6),
    and(7),
    wr(0x58), // 40
    // EN_CBPF & EN_RVGA
    rd(0x4C),
    and(27),
    and(6),
    and(7),
    wr(0x4C),
    // EN_PKDET
    rd(0x50),
    and(0),
    and(1),
    and(2),
    and(3),
    wr(0x50), // 51
    // END should be put in odd
    END, // 52
    END, // 53
];

/// TXON command sequence (BLE TX enable).
static TXON_CMD: [u16; 48] = [
    // VDDPSW/RFBG_EN/LO_IARY_EN
    rd(0x10),
    rd(0x10),
    or(17),
    or(18),
    or(16),
    wr(0x10), // 5
    // WAIT 1US
    wait(2),
    // RD FULCAL
    RD_FULCAL,
    wr(0x08),
    // VCO5G_EN
    rd(0x0),
    or(12),
    wr(0x0), // 11
    // FBDV_EN
    rd(0x14),
    or(12),
    wr(0x14),
    // PFDCP_EN
    rd(0x1C),
    or(19),
    wr(0x1C), // 17
    // FBDV_RSTB
    rd(0x14),
    and(7),
    wr(0x14),
    // wait 30us for lo lock
    wait(30), // 21
    // VCO_FLT_EN
    rd(0x0),
    or(7),
    wr(0x0),
    // LODIST5G_BLETX_EN
    rd(0x10),
    or(8),
    wr(0x10), // 27
    // EDR_IARRAY_EN
    rd(0x3C),
    or(20),
    wr(0x3C),
    // PA_BUF_PU for normal tx
    rd(0x34),
    or(22),
    wr(0x34), // 33
    // EDR_XFMR_SG
    rd(0x40),
    and(11),
    wr(0x40),
    // wait 4us
    wait(5), // 37
    // PA_OUT_PU & TRF_SIG_EN (OR(21): pa_out_pu for normal tx)
    rd(0x34),
    or(16),
    or(21),
    wr(0x34),
    // START INCCAL, 0x74: inccal start
    rd(0x74),
    or(29),
    wr(0x74),
    wait(9), // 45
    // END should be put in odd
    END, // 46
    END, // 47
];

/// TXOFF command sequence (BLE TX disable).
static TXOFF_CMD: [u16; 78] = [
    // VDDPSW /RFBG_EN/LO_IARY_EN/ LODIST5G_BLETX_EN
    rd(0x10),
    rd(0x10),
    and(8),
    and(16),
    and(17),
    and(18),
    wr(0x10), // 6
    // VCO5G_EN & VCO_FLT_EN
    rd(0x0),
    and(12),
    and(7),
    wr(0x00),
    // FBDV_EN / FBDV RSTB
    rd(0x14),
    and(12),
    or(7),
    wr(0x14), // 14
    // PFDCP_EN
    rd(0x1C),
    and(19),
    wr(0x1C), // 17
    // PA_BUF_PU & PA_OUT_PU & TRF_SIG_EN
    rd(0x34),
    and(22),
    and(16),
    and(21),
    wr(0x34),
    // TRF_EDR_IARRAY_EN
    rd(0x3C),
    and(20),
    wr(0x3C), // 25
    // redundancy from bt_txoff
    // DAC_STOP
    // EN_TBB_IARRY & EN_LDO_DAC_AVDD & EN_LDO_DAC_DVDD & EN_DAC
    rd(0x64),
    and(8),
    and(9),
    and(10),
    and(11),
    and(12),
    wr(0x64), // 32
    // EDR_PACAP_EN & EDR_PA_XFMR_SG
    rd(0x40),
    and(11),
    and(17),
    wr(0x40),
    // TRF_EDR_IARRAY_EN
    rd(0x3C),
    and(2),
    and(12),
    and(19),
    wr(0x3C), // 41
    // EDR_EN_OSLO
    rd(0x28),
    and(11),
    wr(0x28),
    // VCO3G_EN/EDR_VCO_FLT_EN
    rd(0x0),
    and(13),
    and(7),
    wr(0x0), // 48
    // EDR_FBDV_RSTB
    rd(0x14),
    or(7),
    wr(0x14),
    // EDR PFDCP_EN
    rd(0x1C),
    and(19),
    wr(0x1C), // 54
    // EDR FBDV_EN/MOD_STG/SDM_CLK_SEL
    rd(0x14),
    and(12),
    or(5),
    and(4),
    or(3),
    wr(0x14),
    // ACAL_VH_SEL=3/ACAL_VL_SEL=1
    rd(0x4),
    and(2),
    and(6),
    wr(0x4), // 64
    // LDO_RBB
    rd(0x48),
    and(13),
    wr(0x48),
    // EDR VCO3G_EN/EDR_VCO5G_EN
    rd(0x00),
    and(13),
    wr(0x24), // 70
    // VDDPSW/ RFBG_EN/ LO_IARY_EN /LODISTEDR_EN
    rd(0x10),
    and(0),
    and(16),
    and(17),
    and(18),
    wr(0x10), // 76
    // END should be put in odd
    END, // 77
];

/// BT TXON command sequence (BT/EDR TX enable).
static BT_TXON_CMD: [u16; 96] = [
    // VDDPSW/RFBG_EN/LO_IARY_EN
    rd(0x10),
    rd(0x10),
    or(16),
    or(17),
    or(18),
    wr(0x10), // 5
    // WAIT 1US
    wait(2),
    // LDO_RBB
    rd(0x48),
    or(13),
    wr(0x48),
    // RD FULCAL
    RD_FULCAL,
    wr(0x24),
    wr(0x6C), // 12
    // VCO3G_EN
    rd(0x0),
    or(13),
    wr(0x0),
    // PFDCP_EN ICP_SET=4/3->1 (OR(11), AND(13))
    rd(0x1C),
    or(19),
    or(11),
    and(13),
    wr(0x1C), // 20
    // FBDV_EN/MOD_STG/SDM_CLK_SEL
    rd(0x14),
    or(12),
    and(5),
    or(4),
    and(3),
    wr(0x14),
    // FBDV_RSTB
    rd(0x14),
    and(7),
    wr(0x14), // 29
    // ACAL_VH_SEL=7/ACAL_VL_SEL=5
    rd(0x04),
    or(2),
    or(6),
    wr(0x04),
    // EDR_VCO_FLT_EN
    rd(0x0),
    or(7),
    wr(0x0), // 36
    // EDR_EN_OSLO
    rd(0x28),
    or(11),
    wr(0x28),
    // LODISTEDR_EN
    rd(0x10),
    or(0),
    wr(0x10), // 42
    // EN_TBB_IARRY & EN_LDO_DAC_AVDD & EN_LDO_DAC_DVDD & EN_DAC
    rd(0x64),
    or(8),
    or(9),
    or(10),
    or(11),
    wr(0x64),
    // TRF_EDR_IARRAY_EN
    rd(0x3C),
    or(20),
    wr(0x3C), // 51
    // EDR_PACAP_EN & EDR_PA_XFMR_SG
    rd(0x40),
    or(11),
    or(17),
    wr(0x40),
    // RD DCCAL
    RD_DCCAL1,
    wr(0xA8),
    RD_DCCAL2,
    wr(0xAC), // 59
    // EDR_TMXBUF_PU EDR_TMX_PU
    rd(0x3C),
    or(12),
    or(19),
    wr(0x3C),
    // cmd for cal
    // RBB_REG5: EN_IARRAY
    rd(0x58),
    or(5),
    wr(0x58), // 66
    // en rvga_i EN_RVGA_I
    rd(0x4C),
    or(7),
    wr(0x4C),
    // adc* ADC & LDO_ADC & LDO_ADCREF
    rd(0x60),
    or(4),
    or(9),
    or(21),
    wr(0x60), // 74
    // wait 5us
    wait(8),
    // pwrmtr_en
    rd(0x40),
    or(10),
    wr(0x40), // 78
    // wait 3us
    wait(5),
    // lpbk en
    rd(0x58),
    or(0),
    wr(0x58), // 82
    // wait 30us for lo lock
    wait(20),
    // START INCCAL
    rd(0x74),
    or(29),
    wr(0x74),
    wait(9),
    // DAC_START
    rd(0x64),
    or(12),
    wr(0x64), // 90
    // EDR_PA_PU
    rd(0x3C),
    or(2),
    wr(0x3C), // 93
    // END should be put in odd
    END, // 94
    END, // 95
];

/// BT TXOFF command sequence (BT/EDR TX disable).
static BT_TXOFF_CMD: [u16; 92] = [
    // EDR_PA_PU
    // EDR_TMXBUF_PU EDR_TMX_PU
    rd(0x3C),
    rd(0x3C),
    and(2),
    and(12),
    and(19),
    wr(0x3C), // 5
    // DAC_STOP
    // EN_TBB_IARRY & EN_LDO_DAC_AVDD & EN_LDO_DAC_DVDD & EN_DAC
    rd(0x64),
    and(8),
    and(9),
    and(10),
    and(11),
    and(12),
    wr(0x64),
    // EDR_PACAP_EN & EDR_PA_XFMR_SG
    rd(0x40),
    and(11),
    and(17),
    wr(0x40), // 16
    // cmd for cal
    // lpbk en
    rd(0x58),
    and(0),
    wr(0x58),
    // wait 1us
    wait(2), // 20
    // pwrmtr_en
    rd(0x40),
    and(10),
    wr(0x40),
    // wait 1us
    wait(2), // 24
    // en iarray EN_IARRAY
    rd(0x58),
    and(5),
    wr(0x58),
    // en rvga_i EN_RVGA_I
    rd(0x4C),
    and(7),
    wr(0x4C), // 30
    // adc* ADC & LDO_ADC & LDO_ADCREF
    rd(0x60),
    and(4),
    and(9),
    and(21),
    wr(0x60),
    // TRF_EDR_IARRAY_EN
    rd(0x3C),
    and(20),
    wr(0x3C), // 38
    // EDR_EN_OSLO
    rd(0x28),
    and(11),
    wr(0x28),
    // VCO3G_EN/EDR_VCO_FLT_EN
    rd(0x0),
    and(13),
    and(7),
    wr(0x0), // 45
    // EDR_FBDV_RSTB
    rd(0x14),
    or(7),
    wr(0x14),
    // EDR PFDCP_EN ICP_SET=1->4/3 (AND(11), OR(13))
    rd(0x1C),
    and(19),
    and(11),
    or(13),
    wr(0x1C), // 53
    // EDR FBDV_EN/MOD_STG/SDM_CLK_SEL
    rd(0x14),
    and(12),
    or(5),
    and(4),
    or(3),
    wr(0x14),
    // ACAL_VH_SEL=3/ACAL_VL_SEL=1
    rd(0x4),
    and(2),
    and(6),
    wr(0x4), // 63
    // LDO_RBB
    rd(0x48),
    and(13),
    wr(0x48),
    // EDR VCO3G_EN/EDR_VCO5G_EN
    rd(0x00),
    and(13),
    wr(0x24), // 69
    // VDDPSW/ RFBG_EN/ LO_IARY_EN /LODISTEDR_EN
    rd(0x10),
    and(0),
    and(16),
    and(17),
    and(18),
    wr(0x10), // 75
    // redundant cmd to fix control change while txoff
    // VCO5G_EN & VCO_FLT_EN
    rd(0x0),
    and(12),
    and(7),
    wr(0x0), // 79
    // FBDV_EN / FBDV_RSTB
    rd(0x14),
    and(12),
    or(7),
    wr(0x14), // 83
    // PFDCP_EN
    rd(0x1C),
    and(19),
    wr(0x1C),
    // PA_BUF_PU & PA_OUT_PU & TRF_SIG_EN
    rd(0x34),
    and(22),
    and(16),
    and(21),
    wr(0x34), // 91
];

/// Initialize RFC and return calibration data start address.
///
/// This function configures the BT_RFC and BT_PHY registers for calibration,
/// and fills the command sequences into RFC memory.
///
/// Returns the memory address where calibration results should be stored.
pub fn rfc_init(edr_mode: EdrMode) -> usize {
    // Enable ADC Q for all PHY modes
    phy_set(RX_CTRL1, ADC_Q_EN_1);
    phy_set(RX_CTRL2, ADC_Q_EN_FRC_EN);

    // Zero IF
    phy_clear(TX_IF_MOD_CFG, TX_IF_PHASE_BLE_MSK);

    // Reset RCCAL
    rfc_clear(RBB_REG5, BRF_RSTB_RCCAL_LV);

    // Release ADC reset
    rfc_set(ADC_REG, BRF_RSTB_ADC_LV);

    // Disable pkdet det early off
    rfc_clear(MISC_CTRL_REG, PKDET_EN_EARLY_OFF_EN);

    // Select VCO for IQ TX based on EDR mode
    match edr_mode {
        EdrMode::Edr5G => {
            phy_clear(TX_CTRL, MMDIV_SEL_MSK);
            // EDR_PLL_REG4.BRF_EDR_SEL_VC_PATH_LV = 1
            rfc_modify(EDR_PLL_REG4, |v| (v & !(0x3 << 0)) | (1 << 0));
            // EDR_OSLO_REG.BRF_EDR_SEL_LODIST_TX_LV = 1
            rfc_modify(EDR_OSLO_REG, |v| (v & !(0x3 << 0)) | (1 << 0));
        }
        EdrMode::Edr3G => {
            // Select 3G VCO for IQ TX
            phy_modify(TX_CTRL, |v| (v & !MMDIV_SEL_MSK) | (0x1 << MMDIV_SEL_POS));
        }
        EdrMode::Edr2G => {
            // Select 2G VCO for IQ TX
            phy_modify(TX_CTRL, |v| (v & !MMDIV_SEL_MSK) | (0x2 << MMDIV_SEL_POS));
        }
    }

    // INCCAL time setting
    rfc_set(
        INCCAL_REG1,
        (0x3f << VCO3G_INCFCAL_WAIT_TIME_POS) | (0x3f << VCO3G_INCACAL_WAIT_TIME_POS),
    );
    rfc_set(
        INCCAL_REG2,
        (0x3f << VCO5G_INCFCAL_WAIT_TIME_POS) | (0x3f << VCO5G_INCACAL_WAIT_TIME_POS),
    );
    rfc_clear(INCCAL_REG1, FRC_INCCAL_CLK_ON);

    // Fill command sequences into RFC memory
    let mut offset = 0usize;

    // RXON commands
    let rxon_addr = offset;
    offset = rfc_cmd_fill(offset, &RXON_CMD);

    // RXOFF commands
    let rxoff_addr = offset;
    offset = rfc_cmd_fill(offset, &RXOFF_CMD);

    // TXON commands
    let txon_addr = offset;
    offset = rfc_cmd_fill(offset, &TXON_CMD);

    // TXOFF commands
    let txoff_addr = offset;
    offset = rfc_cmd_fill(offset, &TXOFF_CMD);

    // BT TXON commands
    let bt_txon_addr = offset;
    offset = rfc_cmd_fill(offset, &BT_TXON_CMD);

    // BT TXOFF commands
    let bt_txoff_addr = offset;
    offset = rfc_cmd_fill(offset, &BT_TXOFF_CMD);

    // Configure command unit addresses
    // CU_ADDR_REG1: rxon (low 16) | rxoff (high 16)
    rfc_write(
        CU_ADDR_REG1,
        (rxon_addr as u32) | ((rxoff_addr as u32) << 16),
    );
    // CU_ADDR_REG2: txon (low 16) | txoff (high 16)
    rfc_write(
        CU_ADDR_REG2,
        (txon_addr as u32) | ((txoff_addr as u32) << 16),
    );
    // CU_ADDR_REG3: bt_txon (low 16) | bt_txoff (high 16)
    rfc_write(
        CU_ADDR_REG3,
        (bt_txon_addr as u32) | ((bt_txoff_addr as u32) << 16),
    );

    // Return the address where calibration results should be stored
    offset
}
