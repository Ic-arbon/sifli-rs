//! SF32LB52x EFUSE bank1 factory calibration values.

/// Decoded EFUSE bank1 calibration values.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bank1Calibration {
    pub primary: Bank1Primary,
    pub vol2: Bank1Vol2,
    /// Bit 124: `IS_IO18`.
    pub is_io18: bool,
}

/// Bank1 calibration fields at positions 0..=159.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bank1Primary {
    pub buck_vos_trim: u8,
    pub buck_vos_polar: bool,
    pub hpsys_ldo_vout: u8,
    pub lpsys_ldo_vout: u8,
    pub vret_trim: u8,
    pub ldo18_vref_sel: u8,
    pub vdd33_ldo2_vout: u8,
    pub vdd33_ldo3_vout: u8,
    pub aon_vos_trim: u8,
    pub aon_vos_polar: bool,

    pub adc_vol1_reg: u16,
    pub volt1_100mv: u8,
    pub adc_vol2_reg: u16,
    pub volt2_100mv: u8,
    pub vbat_reg: u16,
    pub vbat_volt_100mv: u8,

    pub prog_v1p2: u8,
    pub cv_vctrl: u8,
    pub cc_mn: u8,
    pub cc_mp: u8,

    pub buck_vos_trim2: u8,
    pub buck_vos_polar2: bool,
    pub hpsys_ldo_vout2: u8,
    pub lpsys_ldo_vout2: u8,

    pub vbat_step: u8,

    pub edr_cal_done: bool,
    pub pa_bm: u8,
    pub dac_lsb_cnt: u8,

    pub tmxcap_flag: bool,
    pub tmxcap_ch78: u8,
    pub tmxcap_ch00: u8,

    pub vref_flag: bool,
    pub vref_reg: u8,
}

/// Bank1 calibration fields at positions 160..=255.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bank1Vol2 {
    pub buck_vos_trim: u8,
    pub buck_vos_polar: bool,
    pub hpsys_ldo_vout: u8,
    pub lpsys_ldo_vout: u8,
    pub vret_trim: u8,

    pub adc_vol1_reg: u16,
    pub volt1_100mv: u8,
    pub adc_vol2_reg: u16,
    pub volt2_100mv: u8,
    pub vbat_reg: u16,
    pub vbat_volt_100mv: u8,

    pub hpsys_ldo_vout2: u8,
    pub edr_cal_flag: bool,
    pub pa_bm: u8,
    pub dac_lsb_cnt: u8,

    pub tmxcap_flag: bool,
    pub tmxcap_ch78: u8,
    pub tmxcap_ch00: u8,
}

impl Bank1Calibration {
    pub(crate) fn decode(words: &[u32; 8]) -> Self {
        Self {
            primary: Bank1Primary::decode(words),
            vol2: Bank1Vol2::decode(words),
            is_io18: get_bits(words, 124, 1) != 0,
        }
    }
}

impl Bank1Primary {
    fn decode(words: &[u32; 8]) -> Self {
        Self {
            buck_vos_trim: get_bits(words, 0, 3) as u8,
            buck_vos_polar: get_bits(words, 3, 1) != 0,
            hpsys_ldo_vout: get_bits(words, 4, 4) as u8,
            lpsys_ldo_vout: get_bits(words, 8, 4) as u8,
            vret_trim: get_bits(words, 12, 4) as u8,
            ldo18_vref_sel: get_bits(words, 16, 4) as u8,
            vdd33_ldo2_vout: get_bits(words, 20, 4) as u8,
            vdd33_ldo3_vout: get_bits(words, 24, 4) as u8,
            aon_vos_trim: get_bits(words, 28, 4) as u8,
            aon_vos_polar: get_bits(words, 31, 1) != 0,
            adc_vol1_reg: get_bits(words, 32, 12) as u16,
            volt1_100mv: get_bits(words, 44, 5) as u8,
            adc_vol2_reg: get_bits(words, 49, 12) as u16,
            volt2_100mv: get_bits(words, 61, 5) as u8,
            vbat_reg: get_bits(words, 66, 12) as u16,
            vbat_volt_100mv: get_bits(words, 78, 6) as u8,
            prog_v1p2: get_bits(words, 84, 4) as u8,
            cv_vctrl: get_bits(words, 88, 6) as u8,
            cc_mn: get_bits(words, 94, 5) as u8,
            cc_mp: get_bits(words, 99, 5) as u8,
            buck_vos_trim2: get_bits(words, 104, 3) as u8,
            buck_vos_polar2: get_bits(words, 107, 1) != 0,
            hpsys_ldo_vout2: get_bits(words, 108, 4) as u8,
            lpsys_ldo_vout2: get_bits(words, 112, 4) as u8,
            vbat_step: get_bits(words, 116, 8) as u8,
            edr_cal_done: get_bits(words, 125, 1) != 0,
            pa_bm: get_bits(words, 126, 2) as u8,
            dac_lsb_cnt: get_bits(words, 128, 2) as u8,
            tmxcap_flag: get_bits(words, 130, 1) != 0,
            tmxcap_ch78: get_bits(words, 131, 4) as u8,
            tmxcap_ch00: get_bits(words, 135, 4) as u8,
            vref_flag: get_bits(words, 139, 1) != 0,
            vref_reg: get_bits(words, 140, 4) as u8,
        }
    }
}

impl Bank1Vol2 {
    fn decode(words: &[u32; 8]) -> Self {
        Self {
            buck_vos_trim: get_bits(words, 160, 3) as u8,
            buck_vos_polar: get_bits(words, 163, 1) != 0,
            hpsys_ldo_vout: get_bits(words, 164, 4) as u8,
            lpsys_ldo_vout: get_bits(words, 168, 4) as u8,
            vret_trim: get_bits(words, 172, 4) as u8,
            adc_vol1_reg: get_bits(words, 176, 12) as u16,
            volt1_100mv: get_bits(words, 188, 5) as u8,
            adc_vol2_reg: get_bits(words, 193, 12) as u16,
            volt2_100mv: get_bits(words, 205, 5) as u8,
            vbat_reg: get_bits(words, 210, 12) as u16,
            vbat_volt_100mv: get_bits(words, 222, 6) as u8,
            hpsys_ldo_vout2: get_bits(words, 228, 4) as u8,
            edr_cal_flag: get_bits(words, 232, 1) != 0,
            pa_bm: get_bits(words, 233, 2) as u8,
            dac_lsb_cnt: get_bits(words, 235, 2) as u8,
            tmxcap_flag: get_bits(words, 237, 1) != 0,
            tmxcap_ch78: get_bits(words, 238, 4) as u8,
            tmxcap_ch00: get_bits(words, 242, 4) as u8,
        }
    }
}

pub(crate) fn get_bits(words: &[u32; 8], pos: u16, bits: u8) -> u32 {
    debug_assert!(bits >= 1);
    debug_assert!(bits <= 32);

    let pos = pos as usize;
    let bits = bits as usize;

    let word_index = pos / 32;
    let bit_index = pos % 32;
    debug_assert!(word_index < words.len());

    let mut val = words[word_index] >> bit_index;
    let bits_in_low = 32 - bit_index;
    if bits > bits_in_low {
        debug_assert!(word_index + 1 < words.len());
        val |= words[word_index + 1] << bits_in_low;
    }

    if bits == 32 {
        val
    } else {
        val & ((1u32 << bits) - 1)
    }
}

