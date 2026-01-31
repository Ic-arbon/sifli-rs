//! Raw register access for BT_RFC and BT_PHY peripherals.
//!
//! This module provides temporary raw address access to BT_RFC and BT_PHY registers.
//! These registers are not yet included in the sifli-pac, so we use direct memory access.
//!
//! Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/bt_rfc.h` and `bt_phy.h`

use core::ptr::{read_volatile, write_volatile};

// =============================================================================
// Base addresses
// =============================================================================

/// BT_RFC register base address.
pub const BT_RFC_REG_BASE: usize = 0x4008_2800;

/// BT_RFC memory base (for command sequences).
pub const BT_RFC_MEM_BASE: usize = 0x4008_2000;

/// BT_PHY register base address.
pub const BT_PHY_BASE: usize = 0x4008_4000;

// =============================================================================
// RFC Command encoding
// =============================================================================

/// RFC command: Read register at offset n.
#[inline]
pub const fn rd(n: u16) -> u16 {
    0x1800 + n
}

/// RFC command: Write to register at offset n.
#[inline]
pub const fn wr(n: u16) -> u16 {
    0x2800 + n
}

/// RFC command: AND operation with bit n (clear bit n).
#[inline]
pub const fn and(n: u16) -> u16 {
    0x3000 + n
}

/// RFC command: OR operation with bit n (set bit n).
#[inline]
pub const fn or(n: u16) -> u16 {
    0x4000 + n
}

/// RFC command: Wait n cycles.
#[inline]
pub const fn wait(n: u16) -> u16 {
    0x5000 + n
}

/// RFC command: Read full calibration result.
pub const RD_FULCAL: u16 = 0x6000;

/// RFC command: Read DC calibration result 1.
pub const RD_DCCAL1: u16 = 0x7000;

/// RFC command: Read DC calibration result 2.
pub const RD_DCCAL2: u16 = 0x8000;

/// RFC command: End of sequence.
pub const END: u16 = 0xF000;

// =============================================================================
// RFC register offsets
// =============================================================================

/// VCO_REG1 offset (0x00)
pub const VCO_REG1: usize = 0x00;
/// VCO_REG2 offset (0x04)
pub const VCO_REG2: usize = 0x04;
/// VCO_REG3 offset (0x08)
pub const VCO_REG3: usize = 0x08;
/// MISC_CTRL_REG offset (0x0C)
pub const MISC_CTRL_REG: usize = 0x0C;
/// RF_LODIST_REG offset (0x10)
pub const RF_LODIST_REG: usize = 0x10;
/// FBDV_REG1 offset (0x14)
pub const FBDV_REG1: usize = 0x14;
/// FBDV_REG2 offset (0x18)
pub const FBDV_REG2: usize = 0x18;
/// PFDCP_REG offset (0x1C)
pub const PFDCP_REG: usize = 0x1C;
/// LPF_REG offset (0x20)
pub const LPF_REG: usize = 0x20;
/// EDR_CAL_REG1 offset (0x24)
pub const EDR_CAL_REG1: usize = 0x24;
/// OSLO_REG offset (0x28)
pub const OSLO_REG: usize = 0x28;
/// ATEST_REG offset (0x2C)
pub const ATEST_REG: usize = 0x2C;
/// DTEST_REG offset (0x30)
pub const DTEST_REG: usize = 0x30;
/// TRF_REG1 offset (0x34)
pub const TRF_REG1: usize = 0x34;
/// TRF_REG2 offset (0x38)
pub const TRF_REG2: usize = 0x38;
/// TRF_EDR_REG1 offset (0x3C)
pub const TRF_EDR_REG1: usize = 0x3C;
/// TRF_EDR_REG2 offset (0x40)
pub const TRF_EDR_REG2: usize = 0x40;
/// RRF_REG offset (0x44)
pub const RRF_REG: usize = 0x44;
/// RBB_REG1 offset (0x48)
pub const RBB_REG1: usize = 0x48;
/// RBB_REG2 offset (0x4C)
pub const RBB_REG2: usize = 0x4C;
/// RBB_REG3 offset (0x50)
pub const RBB_REG3: usize = 0x50;
/// RBB_REG4 offset (0x54)
pub const RBB_REG4: usize = 0x54;
/// RBB_REG5 offset (0x58)
pub const RBB_REG5: usize = 0x58;
/// RBB_REG6 offset (0x5C)
pub const RBB_REG6: usize = 0x5C;
/// ADC_REG offset (0x60)
pub const ADC_REG: usize = 0x60;
/// TBB_REG offset (0x64)
pub const TBB_REG: usize = 0x64;
/// ATSTBUF_REG offset (0x68)
pub const ATSTBUF_REG: usize = 0x68;
/// RSVD_REG1 offset (0x6C)
pub const RSVD_REG1: usize = 0x6C;
/// RSVD_REG2 offset (0x70)
pub const RSVD_REG2: usize = 0x70;
/// INCCAL_REG1 offset (0x74)
pub const INCCAL_REG1: usize = 0x74;
/// INCCAL_REG2 offset (0x78)
pub const INCCAL_REG2: usize = 0x78;
/// ROSCAL_REG1 offset (0x7C)
pub const ROSCAL_REG1: usize = 0x7C;
/// ROSCAL_REG2 offset (0x80)
pub const ROSCAL_REG2: usize = 0x80;
/// RCROSCAL_REG offset (0x84)
pub const RCROSCAL_REG: usize = 0x84;
/// PACAL_REG offset (0x88)
pub const PACAL_REG: usize = 0x88;
/// CU_ADDR_REG1 offset (0x8C)
pub const CU_ADDR_REG1: usize = 0x8C;
/// CU_ADDR_REG2 offset (0x90)
pub const CU_ADDR_REG2: usize = 0x90;
/// CU_ADDR_REG3 offset (0x94)
pub const CU_ADDR_REG3: usize = 0x94;
/// CAL_ADDR_REG1 offset (0x98)
pub const CAL_ADDR_REG1: usize = 0x98;
/// CAL_ADDR_REG2 offset (0x9C)
pub const CAL_ADDR_REG2: usize = 0x9C;
/// CAL_ADDR_REG3 offset (0xA0)
pub const CAL_ADDR_REG3: usize = 0xA0;
/// AGC_REG offset (0xA4)
pub const AGC_REG: usize = 0xA4;
/// TXDC_CAL_REG1 offset (0xA8)
pub const TXDC_CAL_REG1: usize = 0xA8;
/// TXDC_CAL_REG2 offset (0xAC)
pub const TXDC_CAL_REG2: usize = 0xAC;

// Additional RFC registers for EDR
/// EDR_PLL_REG1 offset (0xB0)
pub const EDR_PLL_REG1: usize = 0xB0;
/// EDR_PLL_REG2 offset (0xB4)
pub const EDR_PLL_REG2: usize = 0xB4;
/// EDR_PLL_REG3 offset (0xB8)
pub const EDR_PLL_REG3: usize = 0xB8;
/// EDR_PLL_REG4 offset (0xBC)
pub const EDR_PLL_REG4: usize = 0xBC;
/// EDR_OSLO_REG offset (0xC0)
pub const EDR_OSLO_REG: usize = 0xC0;

// =============================================================================
// BT_PHY register offsets
// =============================================================================

/// RX_CTRL1 offset
pub const RX_CTRL1: usize = 0x00;
/// RX_CTRL2 offset
pub const RX_CTRL2: usize = 0x04;
/// TX_CTRL offset
pub const TX_CTRL: usize = 0xE8;
/// TX_IF_MOD_CFG offset
pub const TX_IF_MOD_CFG: usize = 0x100;
/// TX_IF_MOD_CFG3 offset
pub const TX_IF_MOD_CFG3: usize = 0x108;
/// TX_IF_MOD_CFG5 offset
pub const TX_IF_MOD_CFG5: usize = 0x110;
/// TX_IF_MOD_CFG6 offset
pub const TX_IF_MOD_CFG6: usize = 0x114;
/// TX_IF_MOD_CFG7 offset
pub const TX_IF_MOD_CFG7: usize = 0x118;
/// TX_HFP_CFG offset
pub const TX_HFP_CFG: usize = 0x11C;
/// TX_LFP_CFG offset
pub const TX_LFP_CFG: usize = 0x120;
/// TX_DPSK_CFG1 offset
pub const TX_DPSK_CFG1: usize = 0x128;
/// TX_DPSK_CFG2 offset
pub const TX_DPSK_CFG2: usize = 0x12C;

// =============================================================================
// Bit definitions for RFC registers
// =============================================================================

// VCO_REG1
pub const BRF_VCO_FLT_EN_LV: u32 = 1 << 7;
pub const BRF_VCO5G_EN_LV: u32 = 1 << 12;
pub const BRF_VCO3G_EN_LV: u32 = 1 << 13;
pub const BRF_EN_2M_MOD_LV: u32 = 1 << 0;

// VCO_REG2
pub const BRF_VCO_ACAL_VL_SEL_LV_POS: u32 = 0;
pub const BRF_VCO_ACAL_VL_SEL_LV_MSK: u32 = 0xF << BRF_VCO_ACAL_VL_SEL_LV_POS;
pub const BRF_VCO_ACAL_VH_SEL_LV_POS: u32 = 4;
pub const BRF_VCO_ACAL_VH_SEL_LV_MSK: u32 = 0xF << BRF_VCO_ACAL_VH_SEL_LV_POS;
pub const BRF_VCO_ACAL_EN_LV: u32 = 1 << 8;
pub const BRF_VCO_FKCAL_EN_LV: u32 = 1 << 19;
pub const BRF_VCO5G_ACAL_UP_LV: u32 = 1 << 25;
pub const BRF_VCO5G_ACAL_INCAL_LV: u32 = 1 << 26;
pub const BRF_VCO3G_ACAL_UP_LV: u32 = 1 << 21;
pub const BRF_VCO3G_ACAL_INCAL_LV: u32 = 1 << 22;

// VCO_REG3
pub const BRF_VCO_PDX_LV_POS: u32 = 0;
pub const BRF_VCO_PDX_LV_MSK: u32 = 0xFF << BRF_VCO_PDX_LV_POS;
pub const BRF_VCO_IDAC_LV_POS: u32 = 8;
pub const BRF_VCO_IDAC_LV_MSK: u32 = 0x7F << BRF_VCO_IDAC_LV_POS;
pub const TX_KCAL_POS: u32 = 16;
pub const TX_KCAL_MSK: u32 = 0xFFF << TX_KCAL_POS;

// MISC_CTRL_REG
pub const PDX_FORCE_EN: u32 = 1 << 0;
pub const IDAC_FORCE_EN: u32 = 1 << 1;
pub const XTAL_REF_EN: u32 = 1 << 2;
pub const XTAL_REF_EN_FRC_EN: u32 = 1 << 6;
pub const PKDET_EN_EARLY_OFF_EN: u32 = 1 << 15;
pub const EN_2M_MOD_FRC_EN: u32 = 1 << 27;

// RF_LODIST_REG
pub const BRF_LO_IARY_EN_LV: u32 = 1 << 16;
pub const BRF_EN_RFBG_LV: u32 = 1 << 17;
pub const BRF_EN_VDDPSW_LV: u32 = 1 << 18;

// FBDV_REG1
pub const BRF_FKCAL_CNT_RDY_LV: u32 = 1 << 0;
pub const BRF_FKCAL_CNT_RSTB_LV: u32 = 1 << 1;
pub const BRF_FKCAL_CNT_EN_LV: u32 = 1 << 2;
pub const BRF_SDM_CLK_SEL_LV: u32 = 1 << 3;
pub const BRF_FBDV_MOD_STG_LV_POS: u32 = 4;
pub const BRF_FBDV_MOD_STG_LV_MSK: u32 = 0x3 << BRF_FBDV_MOD_STG_LV_POS;
pub const BRF_FBDV_RSTB_LV: u32 = 1 << 7;
pub const BRF_FBDV_EN_LV: u32 = 1 << 12;

// FBDV_REG2
pub const BRF_FKCAL_CNT_DIVN_LV_POS: u32 = 0;
pub const BRF_FKCAL_CNT_DIVN_LV_MSK: u32 = 0xFFFF << BRF_FKCAL_CNT_DIVN_LV_POS;
pub const BRF_FKCAL_CNT_OP_LV_POS: u32 = 16;
pub const BRF_FKCAL_CNT_OP_LV_MSK: u32 = 0xFFFF << BRF_FKCAL_CNT_OP_LV_POS;

// PFDCP_REG
pub const BRF_PFDCP_EN_LV: u32 = 1 << 19;
pub const BRF_PFDCP_ICP_SET_LV_POS: u32 = 11;
pub const BRF_PFDCP_ICP_SET_LV_MSK: u32 = 0xF << BRF_PFDCP_ICP_SET_LV_POS;

// LPF_REG
pub const BRF_LO_OPEN_LV: u32 = 1 << 14;

// RBB_REG5
pub const BRF_RSTB_RCCAL_LV: u32 = 1 << 8;

// ADC_REG
pub const BRF_RSTB_ADC_LV: u32 = 1 << 4;

// INCCAL_REG1
pub const VCO3G_INCFCAL_WAIT_TIME_POS: u32 = 0;
pub const VCO3G_INCACAL_WAIT_TIME_POS: u32 = 6;
pub const VCO3G_AUTO_INCACAL_EN: u32 = 1 << 24;
pub const VCO3G_AUTO_INCFCAL_EN: u32 = 1 << 25;
pub const FRC_INCCAL_CLK_ON: u32 = 1 << 29;

// INCCAL_REG2
pub const VCO5G_INCFCAL_WAIT_TIME_POS: u32 = 0;
pub const VCO5G_INCACAL_WAIT_TIME_POS: u32 = 6;
pub const VCO5G_AUTO_INCACAL_EN: u32 = 1 << 24;
pub const VCO5G_AUTO_INCFCAL_EN: u32 = 1 << 25;

// TRF_REG1
pub const BRF_PA_PM_LV_POS: u32 = 0;
pub const BRF_PA_PM_LV_MSK: u32 = 0x3 << BRF_PA_PM_LV_POS;
pub const BRF_PA_CAS_BP_LV_POS: u32 = 2;
pub const BRF_PA_CAS_BP_LV_MSK: u32 = 0x1 << BRF_PA_CAS_BP_LV_POS;

// TRF_REG2
pub const BRF_PA_UNIT_SEL_LV_POS: u32 = 0;
pub const BRF_PA_UNIT_SEL_LV_MSK: u32 = 0x1F << BRF_PA_UNIT_SEL_LV_POS;
pub const BRF_PA_MCAP_LV_POS: u32 = 5;
pub const BRF_PA_MCAP_LV_MSK: u32 = 0x3 << BRF_PA_MCAP_LV_POS;

// TRF_EDR_REG1
pub const BRF_TRF_EDR_TMXCAS_SEL_LV: u32 = 1 << 0;

// =============================================================================
// Bit definitions for PHY registers
// =============================================================================

// RX_CTRL1
pub const ADC_Q_EN_1: u32 = 1 << 4;
pub const BT_OP_MODE: u32 = 1 << 26;

// RX_CTRL2
pub const ADC_Q_EN_FRC_EN: u32 = 1 << 25;

// TX_CTRL
pub const MMDIV_SEL_POS: u32 = 0;
pub const MMDIV_SEL_MSK: u32 = 0x3 << MMDIV_SEL_POS;

// TX_IF_MOD_CFG
pub const TX_IF_PHASE_BLE_POS: u32 = 0;
pub const TX_IF_PHASE_BLE_MSK: u32 = 0xFFFF << TX_IF_PHASE_BLE_POS;

// TX_HFP_CFG
pub const HFP_FCW_POS: u32 = 0;
pub const HFP_FCW_MSK: u32 = 0xFFF << HFP_FCW_POS;
pub const HFP_FCW_SEL: u32 = 1 << 12;

// TX_LFP_CFG
pub const LFP_FCW_POS: u32 = 0;
pub const LFP_FCW_MSK: u32 = 0xFFFFF << LFP_FCW_POS;
pub const LFP_FCW_SEL: u32 = 1 << 20;

// =============================================================================
// Memory access functions
// =============================================================================

/// Read a 32-bit value from RFC register.
#[inline]
pub fn rfc_read(offset: usize) -> u32 {
    unsafe { read_volatile((BT_RFC_REG_BASE + offset) as *const u32) }
}

/// Write a 32-bit value to RFC register.
#[inline]
pub fn rfc_write(offset: usize, val: u32) {
    unsafe { write_volatile((BT_RFC_REG_BASE + offset) as *mut u32, val) }
}

/// Modify RFC register (read-modify-write).
#[inline]
pub fn rfc_modify<F: FnOnce(u32) -> u32>(offset: usize, f: F) {
    let val = rfc_read(offset);
    rfc_write(offset, f(val));
}

/// Set bits in RFC register.
#[inline]
pub fn rfc_set(offset: usize, bits: u32) {
    rfc_modify(offset, |v| v | bits);
}

/// Clear bits in RFC register.
#[inline]
pub fn rfc_clear(offset: usize, bits: u32) {
    rfc_modify(offset, |v| v & !bits);
}

/// Read a 32-bit value from PHY register.
#[inline]
pub fn phy_read(offset: usize) -> u32 {
    unsafe { read_volatile((BT_PHY_BASE + offset) as *const u32) }
}

/// Write a 32-bit value to PHY register.
#[inline]
pub fn phy_write(offset: usize, val: u32) {
    unsafe { write_volatile((BT_PHY_BASE + offset) as *mut u32, val) }
}

/// Modify PHY register (read-modify-write).
#[inline]
pub fn phy_modify<F: FnOnce(u32) -> u32>(offset: usize, f: F) {
    let val = phy_read(offset);
    phy_write(offset, f(val));
}

/// Set bits in PHY register.
#[inline]
pub fn phy_set(offset: usize, bits: u32) {
    phy_modify(offset, |v| v | bits);
}

/// Clear bits in PHY register.
#[inline]
pub fn phy_clear(offset: usize, bits: u32) {
    phy_modify(offset, |v| v & !bits);
}

/// Write a 32-bit value to RFC memory (for command sequences).
#[inline]
pub fn rfc_mem_write(offset: usize, val: u32) {
    unsafe { write_volatile((BT_RFC_MEM_BASE + offset) as *mut u32, val) }
}

/// Read a 32-bit value from RFC memory.
#[inline]
pub fn rfc_mem_read(offset: usize) -> u32 {
    unsafe { read_volatile((BT_RFC_MEM_BASE + offset) as *const u32) }
}

/// Fill RFC command sequence to memory.
///
/// Returns the next available offset.
pub fn rfc_cmd_fill(mut offset: usize, cmds: &[u16]) -> usize {
    for chunk in cmds.chunks(2) {
        let cmd = if chunk.len() == 2 {
            (chunk[0] as u32) | ((chunk[1] as u32) << 16)
        } else {
            chunk[0] as u32
        };
        rfc_mem_write(offset, cmd);
        offset += 4;
    }
    offset
}
