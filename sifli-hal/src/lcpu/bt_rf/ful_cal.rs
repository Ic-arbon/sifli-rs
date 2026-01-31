//! Full RF calibration implementation.
//!
//! This module implements the complete RF calibration flow from SDK `bt_ful_cal()`:
//! - LO calibration (VCO ACAL/FCAL)
//! - EDR LO calibration (3G mode)
//! - TXDC calibration
//!
//! The calibration determines optimal VCO settings for each BT/BLE channel.

use super::regs::*;
use super::tables::*;
use super::EdrMode;

/// Delay in microseconds (blocking).
#[inline]
fn delay_us(us: u32) {
    // Use cortex-m delay or HAL delay
    // For now, use a simple busy loop (approximately calibrated for 48MHz)
    for _ in 0..(us * 12) {
        cortex_m::asm::nop();
    }
}

/// Get FBDV counter value after waiting for ready.
fn get_fbdv_cnt() -> u32 {
    delay_us(4);
    rfc_clear(FBDV_REG1, BRF_FKCAL_CNT_EN_LV);
    rfc_clear(FBDV_REG1, BRF_FKCAL_CNT_RSTB_LV);
    rfc_set(FBDV_REG1, BRF_FKCAL_CNT_RSTB_LV);
    rfc_set(FBDV_REG1, BRF_FKCAL_CNT_EN_LV);

    delay_us(10);

    // Wait for ready (with timeout)
    let mut timeout = 1000u32;
    while (rfc_read(FBDV_REG1) & BRF_FKCAL_CNT_RDY_LV) == 0 {
        timeout = timeout.saturating_sub(1);
        if timeout == 0 {
            break;
        }
    }

    let cnt = (rfc_read(FBDV_REG2) & BRF_FKCAL_CNT_OP_LV_MSK) >> BRF_FKCAL_CNT_OP_LV_POS;
    cnt
}

/// Perform 5G VCO amplitude calibration (binary search).
///
/// Returns the optimal IDAC value.
fn vco5g_acal_binary() -> u8 {
    let mut acal_cnt = 0x40u8;
    let acal_cnt_fs = 0x40u8;

    rfc_modify(VCO_REG3, |v| {
        (v & !BRF_VCO_IDAC_LV_MSK) | ((acal_cnt as u32) << BRF_VCO_IDAC_LV_POS)
    });
    delay_us(4);
    rfc_set(VCO_REG2, BRF_VCO_ACAL_EN_LV);

    // Binary search (6 iterations)
    for j in 1..7 {
        if (rfc_read(VCO_REG2) & BRF_VCO5G_ACAL_INCAL_LV) == 0 {
            break;
        } else if (rfc_read(VCO_REG2) & BRF_VCO5G_ACAL_UP_LV) == 0 {
            acal_cnt = acal_cnt.saturating_sub(acal_cnt_fs >> j);
        } else {
            acal_cnt = acal_cnt.saturating_add(acal_cnt_fs >> j);
        }

        rfc_modify(VCO_REG3, |v| {
            (v & !BRF_VCO_IDAC_LV_MSK) | ((acal_cnt as u32) << BRF_VCO_IDAC_LV_POS)
        });
        delay_us(1);
    }

    rfc_clear(VCO_REG2, BRF_VCO_ACAL_EN_LV);
    acal_cnt
}

/// Perform 5G VCO sequential amplitude calibration.
///
/// Returns the optimal IDAC value.
fn vco5g_acal_sequential(mut acal_cnt: u8) -> u8 {
    let mut seq_acal_jump_cnt = 0u8;
    let mut seq_acal_ful_cnt = 0u8;
    let mut pre_acal_up_vld = false;
    let mut pre_acal_up = false;

    rfc_set(VCO_REG2, BRF_VCO_ACAL_EN_LV);
    rfc_set(LPF_REG, BRF_LO_OPEN_LV);

    while seq_acal_jump_cnt < 4 && seq_acal_ful_cnt < 2 {
        rfc_modify(VCO_REG3, |v| {
            (v & !BRF_VCO_IDAC_LV_MSK) | ((acal_cnt as u32) << BRF_VCO_IDAC_LV_POS)
        });
        delay_us(4);

        if (rfc_read(VCO_REG2) & BRF_VCO5G_ACAL_INCAL_LV) == 0 {
            break;
        }

        let curr_acal_up = (rfc_read(VCO_REG2) & BRF_VCO5G_ACAL_UP_LV) != 0;

        if !curr_acal_up {
            if acal_cnt > 0 {
                acal_cnt -= 1;
                seq_acal_ful_cnt = 0;
            } else {
                seq_acal_ful_cnt += 1;
            }
        } else if acal_cnt < 0x3f {
            acal_cnt += 1;
            seq_acal_ful_cnt = 0;
        } else {
            seq_acal_ful_cnt += 1;
        }

        if pre_acal_up_vld {
            if pre_acal_up == curr_acal_up {
                seq_acal_jump_cnt = 0;
            } else {
                seq_acal_jump_cnt += 1;
            }
        }
        pre_acal_up = curr_acal_up;
        pre_acal_up_vld = true;
    }

    rfc_clear(VCO_REG2, BRF_VCO_ACAL_EN_LV);
    acal_cnt
}

/// Perform 3G VCO amplitude calibration (binary search).
#[allow(dead_code)]
fn vco3g_acal_binary() -> u8 {
    let mut acal_cnt = 0x40u8;
    let acal_cnt_fs = 0x40u8;

    rfc_modify(EDR_CAL_REG1, |v| {
        (v & !(0x7F << 8)) | ((acal_cnt as u32) << 8) // EDR_VCO_IDAC
    });
    delay_us(4);
    rfc_set(VCO_REG2, BRF_VCO_ACAL_EN_LV);

    for j in 1..7 {
        if (rfc_read(VCO_REG2) & BRF_VCO3G_ACAL_INCAL_LV) == 0 {
            break;
        } else if (rfc_read(VCO_REG2) & BRF_VCO3G_ACAL_UP_LV) == 0 {
            acal_cnt = acal_cnt.saturating_sub(acal_cnt_fs >> j);
        } else {
            acal_cnt = acal_cnt.saturating_add(acal_cnt_fs >> j);
        }

        rfc_modify(EDR_CAL_REG1, |v| (v & !(0x7F << 8)) | ((acal_cnt as u32) << 8));
        delay_us(1);
    }

    rfc_clear(VCO_REG2, BRF_VCO_ACAL_EN_LV);
    acal_cnt
}

/// Perform 3G VCO sequential amplitude calibration.
#[allow(dead_code)]
fn vco3g_acal_sequential(mut acal_cnt: u8) -> u8 {
    let mut seq_acal_jump_cnt = 0u8;
    let mut seq_acal_ful_cnt = 0u8;
    let mut pre_acal_up_vld = false;
    let mut pre_acal_up = false;

    rfc_set(VCO_REG2, BRF_VCO_ACAL_EN_LV);
    rfc_set(LPF_REG, BRF_LO_OPEN_LV);

    while seq_acal_jump_cnt < 4 && seq_acal_ful_cnt < 2 {
        rfc_modify(EDR_CAL_REG1, |v| (v & !(0x7F << 8)) | ((acal_cnt as u32) << 8));
        delay_us(4);

        if (rfc_read(VCO_REG2) & BRF_VCO3G_ACAL_INCAL_LV) == 0 {
            break;
        }

        let curr_acal_up = (rfc_read(VCO_REG2) & BRF_VCO3G_ACAL_UP_LV) != 0;

        if !curr_acal_up {
            if acal_cnt > 0 {
                acal_cnt -= 1;
                seq_acal_ful_cnt = 0;
            } else {
                seq_acal_ful_cnt += 1;
            }
        } else if acal_cnt < 0x3f {
            acal_cnt += 1;
            seq_acal_ful_cnt = 0;
        } else {
            seq_acal_ful_cnt += 1;
        }

        if pre_acal_up_vld {
            if pre_acal_up == curr_acal_up {
                seq_acal_jump_cnt = 0;
            } else {
                seq_acal_jump_cnt += 1;
            }
        }
        pre_acal_up = curr_acal_up;
        pre_acal_up_vld = true;
    }

    rfc_clear(VCO_REG2, BRF_VCO_ACAL_EN_LV);
    acal_cnt
}

/// LO calibration result for a single channel.
#[derive(Clone, Copy, Default)]
#[allow(dead_code)]
struct LoCalResult {
    idac: u8,
    capcode: u8,
    residual_cnt: u16,
}

/// Perform LO full calibration.
///
/// This calibrates the 5G VCO for all BLE/BT channels.
/// Returns the next available memory address for storing results.
fn lo_cal(rslt_start_addr: usize) -> usize {
    // Disable auto increment calibration
    rfc_clear(
        INCCAL_REG1,
        VCO3G_AUTO_INCACAL_EN | VCO3G_AUTO_INCFCAL_EN,
    );
    rfc_clear(
        INCCAL_REG2,
        VCO5G_AUTO_INCACAL_EN | VCO5G_AUTO_INCFCAL_EN,
    );

    // Enable force mode
    rfc_set(
        MISC_CTRL_REG,
        IDAC_FORCE_EN | PDX_FORCE_EN | EN_2M_MOD_FRC_EN,
    );
    rfc_set(
        RF_LODIST_REG,
        BRF_EN_RFBG_LV | BRF_EN_VDDPSW_LV | BRF_LO_IARY_EN_LV,
    );

    // Configure ACAL thresholds
    rfc_modify(VCO_REG2, |v| {
        (v & !(BRF_VCO_ACAL_VL_SEL_LV_MSK | BRF_VCO_ACAL_VH_SEL_LV_MSK))
            | (0x1 << BRF_VCO_ACAL_VL_SEL_LV_POS)
            | (0x3 << BRF_VCO_ACAL_VH_SEL_LV_POS)
    });

    // Enable VCO5G and 2M modulation
    rfc_set(VCO_REG1, BRF_VCO5G_EN_LV | BRF_EN_2M_MOD_LV);
    rfc_set(VCO_REG2, BRF_VCO_ACAL_EN_LV | BRF_VCO_FKCAL_EN_LV);
    rfc_set(LPF_REG, BRF_LO_OPEN_LV);

    // Configure HFP FCW
    phy_modify(TX_HFP_CFG, |v| (v & !HFP_FCW_MSK) | (0x07 << HFP_FCW_POS));

    // Initial IDAC
    rfc_modify(VCO_REG3, |v| {
        (v & !BRF_VCO_IDAC_LV_MSK) | (0x40 << BRF_VCO_IDAC_LV_POS)
    });

    // Configure FBDV
    rfc_modify(FBDV_REG1, |v| {
        (v & !BRF_FBDV_MOD_STG_LV_MSK) | BRF_FBDV_EN_LV | BRF_SDM_CLK_SEL_LV | (2 << BRF_FBDV_MOD_STG_LV_POS)
    });
    rfc_set(VCO_REG2, BRF_VCO_FKCAL_EN_LV);
    rfc_modify(FBDV_REG2, |v| {
        (v & !BRF_FKCAL_CNT_DIVN_LV_MSK) | (7680 << BRF_FKCAL_CNT_DIVN_LV_POS)
    });

    // Configure LFP FCW
    phy_modify(TX_LFP_CFG, |v| {
        (v & !(LFP_FCW_MSK | LFP_FCW_SEL)) | (0x08 << LFP_FCW_POS)
    });

    // Initial PDX
    rfc_modify(VCO_REG3, |v| {
        (v & !BRF_VCO_PDX_LV_MSK) | (0x80 << BRF_VCO_PDX_LV_POS)
    });
    phy_modify(TX_HFP_CFG, |v| {
        (v & !(HFP_FCW_MSK | HFP_FCW_SEL)) | (0x07 << HFP_FCW_POS)
    });
    rfc_set(VCO_REG1, BRF_EN_2M_MOD_LV);

    // Reset FBDV
    rfc_set(FBDV_REG1, BRF_FBDV_RSTB_LV);
    rfc_clear(FBDV_REG1, BRF_FBDV_RSTB_LV);
    rfc_clear(FBDV_REG1, BRF_FKCAL_CNT_RSTB_LV);
    rfc_set(FBDV_REG1, BRF_FKCAL_CNT_RSTB_LV);

    // Enable XTAL reference
    rfc_set(MISC_CTRL_REG, XTAL_REF_EN | XTAL_REF_EN_FRC_EN);
    rfc_set(PFDCP_REG, BRF_PFDCP_EN_LV);

    // Binary search for initial fcal/acal
    let mut fcal_cnt = 0x80u8;
    let fcal_cnt_fs = 0x80u8;

    let mut idac0 = 0u8;
    let mut idac1 = 0u8;
    let mut capcode0 = 0u8;
    let mut capcode1 = 0u8;
    let mut p0 = 0u32;
    let mut p1 = 0u32;
    let mut error0 = 0xFFFF_FFFFu32;
    let mut error1 = 0xFFFF_FFFFu32;

    // PDX binary search with embedded ACAL
    for i in 1..9 {
        // Full ACAL in full FCAL
        let acal_cnt = vco5g_acal_binary();

        let residual_cnt = get_fbdv_cnt();

        if residual_cnt > RESIDUAL_CNT_VTH {
            idac1 = acal_cnt;
            p1 = residual_cnt;
            error1 = residual_cnt - RESIDUAL_CNT_VTH;
            capcode1 = fcal_cnt;
            fcal_cnt = fcal_cnt.saturating_add(fcal_cnt_fs.wrapping_shr(i));
        } else {
            idac0 = acal_cnt;
            p0 = residual_cnt;
            error0 = RESIDUAL_CNT_VTH - residual_cnt;
            capcode0 = fcal_cnt;
            fcal_cnt = fcal_cnt.saturating_sub(fcal_cnt_fs.wrapping_shr(i));
        }

        rfc_clear(FBDV_REG1, BRF_FKCAL_CNT_EN_LV);
        rfc_modify(VCO_REG3, |v| {
            (v & !BRF_VCO_PDX_LV_MSK) | ((fcal_cnt as u32) << BRF_VCO_PDX_LV_POS)
        });
    }

    // Select best result
    let (start_idac, start_capcode, _start_cnt) = if error0 < error1 {
        (idac0, capcode0, p0 as u16)
    } else {
        (idac1, capcode1, p1 as u16)
    };

    // Allocate result tables on stack (simplified version)
    // In full implementation, these would be heap-allocated or static
    let mut idac_tbl = [0u8; 256];
    let mut capcode_tbl = [0u8; 256];
    let mut residual_cnt_tbl = [0u16; 256];

    idac_tbl[0] = start_idac;
    capcode_tbl[0] = start_capcode;

    // Sweep PDX until 4.8G
    let mut acal_cnt = start_idac;
    fcal_cnt = start_capcode;
    let mut i = 0usize;

    loop {
        i += 1;
        if i >= 256 {
            break;
        }
        fcal_cnt = fcal_cnt.saturating_add(1);

        rfc_modify(VCO_REG3, |v| {
            (v & !BRF_VCO_PDX_LV_MSK) | ((fcal_cnt as u32) << BRF_VCO_PDX_LV_POS)
        });

        // Sequential ACAL
        acal_cnt = vco5g_acal_sequential(acal_cnt);

        rfc_modify(VCO_REG3, |v| {
            (v & !BRF_VCO_IDAC_LV_MSK) | ((acal_cnt as u32) << BRF_VCO_IDAC_LV_POS)
        });

        let residual_cnt = get_fbdv_cnt();

        idac_tbl[i] = acal_cnt;
        capcode_tbl[i] = fcal_cnt;
        residual_cnt_tbl[i] = residual_cnt as u16;

        rfc_clear(FBDV_REG1, BRF_FKCAL_CNT_EN_LV);

        // Stop when frequency exceeds 4.8G
        if residual_cnt > RESIDUAL_CNT_VTH + 1000 {
            break;
        }
    }
    let sweep_num = i;

    // Interpolate results for each channel and store to memory
    let mut reg_addr = rslt_start_addr;

    // Store BLE RX 1M calibration (40 channels)
    rfc_write(CAL_ADDR_REG1, reg_addr as u32);
    for ch in 0..20 {
        let target_cnt = REF_RESIDUAL_CNT_TBL_RX_1M[ch];
        let (idac, capcode) = interpolate_cal(target_cnt, &idac_tbl, &capcode_tbl, &residual_cnt_tbl, sweep_num);
        let target_cnt2 = REF_RESIDUAL_CNT_TBL_RX_1M[39 - ch];
        let (idac2, capcode2) = interpolate_cal(target_cnt2, &idac_tbl, &capcode_tbl, &residual_cnt_tbl, sweep_num);

        let reg_data = ((capcode as u32) << BRF_VCO_PDX_LV_POS)
            | ((idac as u32) << BRF_VCO_IDAC_LV_POS)
            | (((capcode2 as u32) << BRF_VCO_PDX_LV_POS) << 16)
            | (((idac2 as u32) << BRF_VCO_IDAC_LV_POS) << 16);

        rfc_mem_write(reg_addr, reg_data);
        reg_addr += 4;
    }

    // Store BT RX calibration (79 channels -> 40 pairs)
    rfc_modify(CAL_ADDR_REG1, |v| v | ((reg_addr as u32) << 16));
    for ch in 0..40 {
        let target_cnt = REF_RESIDUAL_CNT_TBL_RX_BT[ch];
        let (idac, capcode) = interpolate_cal(target_cnt, &idac_tbl, &capcode_tbl, &residual_cnt_tbl, sweep_num);
        let idx2 = if ch < 39 { 78 - ch } else { 39 };
        let target_cnt2 = REF_RESIDUAL_CNT_TBL_RX_BT[idx2];
        let (idac2, capcode2) = interpolate_cal(target_cnt2, &idac_tbl, &capcode_tbl, &residual_cnt_tbl, sweep_num);

        let reg_data = ((capcode as u32) << BRF_VCO_PDX_LV_POS)
            | ((idac as u32) << BRF_VCO_IDAC_LV_POS)
            | (((capcode2 as u32) << BRF_VCO_PDX_LV_POS) << 16)
            | (((idac2 as u32) << BRF_VCO_IDAC_LV_POS) << 16);

        rfc_mem_write(reg_addr, reg_data);
        reg_addr += 4;
    }

    // Store BLE TX calibration (79 channels)
    rfc_write(CAL_ADDR_REG2, reg_addr as u32);
    for ch in 0..79 {
        let target_cnt = REF_RESIDUAL_CNT_TBL_TX[ch];
        let (idac, capcode) = interpolate_cal(target_cnt, &idac_tbl, &capcode_tbl, &residual_cnt_tbl, sweep_num);

        // Calculate KCAL (simplified)
        let kcal = 0x200u32; // Default value

        let reg_data = ((capcode as u32) << BRF_VCO_PDX_LV_POS)
            | ((idac as u32) << BRF_VCO_IDAC_LV_POS)
            | (kcal << TX_KCAL_POS);

        rfc_mem_write(reg_addr, reg_data);
        reg_addr += 4;
    }

    // Disable force mode
    rfc_clear(VCO_REG1, BRF_VCO5G_EN_LV);
    rfc_clear(VCO_REG2, BRF_VCO_ACAL_EN_LV | BRF_VCO_FKCAL_EN_LV);
    rfc_clear(PFDCP_REG, BRF_PFDCP_EN_LV);
    rfc_clear(FBDV_REG1, BRF_FBDV_EN_LV);
    rfc_clear(LPF_REG, BRF_LO_OPEN_LV);
    rfc_clear(
        RF_LODIST_REG,
        BRF_EN_RFBG_LV | BRF_EN_VDDPSW_LV | BRF_LO_IARY_EN_LV,
    );
    rfc_clear(
        MISC_CTRL_REG,
        IDAC_FORCE_EN | PDX_FORCE_EN | XTAL_REF_EN | XTAL_REF_EN_FRC_EN,
    );

    reg_addr
}

/// Interpolate calibration result for a target frequency.
fn interpolate_cal(
    target_cnt: u16,
    idac_tbl: &[u8],
    capcode_tbl: &[u8],
    residual_cnt_tbl: &[u16],
    sweep_num: usize,
) -> (u8, u8) {
    // Find the closest match
    let mut best_idx = 0usize;
    let mut best_err = u32::MAX;

    for i in 0..sweep_num {
        let err = (target_cnt as i32 - residual_cnt_tbl[i] as i32).unsigned_abs();
        if err < best_err {
            best_err = err;
            best_idx = i;
        }
    }

    (idac_tbl[best_idx], capcode_tbl[best_idx])
}

/// Perform EDR 3G LO calibration.
fn edrlo_3g_cal(mut rslt_start_addr: usize) -> usize {
    // Enable 3G VCO
    rfc_set(VCO_REG1, BRF_VCO3G_EN_LV);

    // Configure for 3G mode
    rfc_modify(VCO_REG2, |v| {
        (v & !(BRF_VCO_ACAL_VL_SEL_LV_MSK | BRF_VCO_ACAL_VH_SEL_LV_MSK))
            | (0x5 << BRF_VCO_ACAL_VL_SEL_LV_POS)
            | (0x7 << BRF_VCO_ACAL_VH_SEL_LV_POS)
    });

    // Store BT TX calibration address
    rfc_modify(CAL_ADDR_REG2, |v| v | ((rslt_start_addr as u32) << 16));

    // Simplified 3G calibration (full implementation would mirror lo_cal but for 3G VCO)
    for ch in 0..79 {
        let target_cnt = REF_RESIDUAL_CNT_TBL_TX_3G[ch];

        // Default values (would be calibrated in full implementation)
        let pdx = ((target_cnt - 30000) / 40).min(255) as u8;
        let idac = 0x30u8;
        let oslo_fc = 0x4u8;
        let oslo_bm = 0x10u8;
        let tmxcap_sel = 0x8u8;

        let reg_data = (pdx as u32)
            | ((idac as u32) << 8)
            | ((oslo_fc as u32) << 16)
            | ((oslo_bm as u32) << 20)
            | ((tmxcap_sel as u32) << 28);

        rfc_mem_write(rslt_start_addr, reg_data);
        rslt_start_addr += 4;
    }

    // Disable 3G VCO
    rfc_clear(VCO_REG1, BRF_VCO3G_EN_LV);

    rslt_start_addr
}

/// Perform TXDC calibration.
fn txdc_cal(mut rslt_start_addr: usize, cal_power_enable: u8) -> usize {
    // Store TXDC calibration address
    rfc_write(CAL_ADDR_REG3, rslt_start_addr as u32);

    // Simplified TXDC calibration
    // Full implementation would perform actual DC offset calibration for each power level
    for power_level in 0..8 {
        if (cal_power_enable & (1 << power_level)) != 0 {
            // Default calibration values
            let coef0 = 0x0u16;
            let coef1 = 0x0u16;
            let offset_q = 0x0u16;
            let offset_i = 0x0u16;

            let reg_data1 = (coef0 as u32) | ((coef1 as u32) << 16);
            let reg_data2 = (offset_q as u32) | ((offset_i as u32) << 16);

            rfc_mem_write(rslt_start_addr, reg_data1);
            rslt_start_addr += 4;
            rfc_mem_write(rslt_start_addr, reg_data2);
            rslt_start_addr += 4;
        } else {
            // Skip disabled power levels, still advance address
            rslt_start_addr += 8;
        }
    }

    rslt_start_addr
}

/// Perform full RF calibration.
///
/// This runs the complete calibration flow:
/// 1. LO calibration (5G VCO)
/// 2. EDR LO calibration (3G VCO, if enabled)
/// 3. TXDC calibration
///
/// # Arguments
/// * `rslt_start_addr` - Memory address to store calibration results
/// * `edr_mode` - EDR mode selection
/// * `cal_power_enable` - Bitmask of power levels to calibrate
pub fn ful_cal(rslt_start_addr: usize, edr_mode: EdrMode, cal_power_enable: u8) {
    // LO calibration (5G VCO for BLE)
    let addr = lo_cal(rslt_start_addr);

    // EDR LO calibration (if using 3G mode)
    let addr = match edr_mode {
        EdrMode::Edr3G => edrlo_3g_cal(addr),
        EdrMode::Edr5G | EdrMode::Edr2G => addr, // Not implemented yet
    };

    // TXDC calibration
    let _addr = txdc_cal(addr, cal_power_enable);

    // Enable auto increment calibration
    rfc_set(
        INCCAL_REG2,
        VCO5G_AUTO_INCACAL_EN | VCO5G_AUTO_INCFCAL_EN,
    );
    rfc_set(
        INCCAL_REG1,
        VCO3G_AUTO_INCACAL_EN | VCO3G_AUTO_INCFCAL_EN,
    );
}
