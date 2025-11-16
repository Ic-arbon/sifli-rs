use crate::pac::LPSYS_RCC;
use crate::rcc::{get_hxt48_freq, get_hrc48_freq};
use crate::time::Hertz;

// 直接复用 PAC 定义的枚举，保持与寄存器位一致。
pub use crate::pac::lpsys_rcc::vals::{
    SelPeri as ClkPeriSel,
    SelSys as ClkSysSel,
    SelTick as TickSel,
};

/// LPSYS 主系统时钟选择（高层视图）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LpsysSysClkSel {
    /// 48MHz 内部 RC 振荡器
    Hrc48,
    /// 48MHz 外部晶振
    Hxt48,
    /// 低速 WDT 域时钟
    Wdt,
}

#[cfg(feature = "defmt")]
impl defmt::Format for LpsysSysClkSel {
    fn format(&self, f: defmt::Formatter) {
        match self {
            LpsysSysClkSel::Hrc48 => defmt::write!(f, "Hrc48"),
            LpsysSysClkSel::Hxt48 => defmt::write!(f, "Hxt48"),
            LpsysSysClkSel::Wdt => defmt::write!(f, "Wdt"),
        }
    }
}

fn get_clk_wdt_freq() -> Option<Hertz> {
    // TODO: 根据实际硬件实现准确的 WDT 时钟频率。
    // 这里先与 lcpu::check_lcpu_frequency 中使用的近似值保持一致。
    Some(Hertz(32_768))
}

/// 获取 LPSYS 主时钟源（包含 WDT 域）
pub fn get_clk_lpsys_source() -> LpsysSysClkSel {
    let csr = LPSYS_RCC.csr().read();

    if csr.sel_sys_lp() {
        LpsysSysClkSel::Wdt
    } else {
        match csr.sel_sys() {
            ClkSysSel::Hrc48 => LpsysSysClkSel::Hrc48,
            ClkSysSel::Hxt48 => LpsysSysClkSel::Hxt48,
        }
    }
}

/// 获取 LPSYS 主时钟频率（clk_lpsys）
pub fn get_clk_lpsys_freq() -> Option<Hertz> {
    match get_clk_lpsys_source() {
        LpsysSysClkSel::Hrc48 => get_hrc48_freq(),
        LpsysSysClkSel::Hxt48 => get_hxt48_freq(),
        LpsysSysClkSel::Wdt => get_clk_wdt_freq(),
    }
}

/// 获取 LPSYS HCLK 分频（HDIV1）
pub fn get_hclk_lpsys_div() -> u8 {
    LPSYS_RCC.cfgr().read().hdiv1()
}

/// 获取 LPSYS HCLK 频率（hclk_lpsys）
pub fn get_hclk_lpsys_freq() -> Option<Hertz> {
    let clk_lpsys = get_clk_lpsys_freq()?;
    let div = get_hclk_lpsys_div();

    if div == 0 {
        Some(clk_lpsys)
    } else {
        Some(clk_lpsys / div as u32)
    }
}

/// 获取 LPSYS PCLK1 分频（PDIV1）
pub fn get_pclk1_lpsys_div() -> u8 {
    LPSYS_RCC.cfgr().read().pdiv1()
}

/// 获取 LPSYS PCLK2 分频（PDIV2）
pub fn get_pclk2_lpsys_div() -> u8 {
    LPSYS_RCC.cfgr().read().pdiv2()
}

/// 获取 LPSYS PCLK1 频率
pub fn get_pclk1_lpsys_freq() -> Option<Hertz> {
    let hclk = get_hclk_lpsys_freq()?;
    Some(hclk / (1 << get_pclk1_lpsys_div()) as u32)
}

/// 获取 LPSYS PCLK2 频率
pub fn get_pclk2_lpsys_freq() -> Option<Hertz> {
    let hclk = get_hclk_lpsys_freq()?;
    Some(hclk / (1 << get_pclk2_lpsys_div()) as u32)
}

/// 获取 LPSYS 外设时钟源（clk_peri_lpsys）
pub fn get_clk_peri_lpsys_source() -> ClkPeriSel {
    LPSYS_RCC.csr().read().sel_peri()
}

/// 获取 LPSYS 外设时钟频率（clk_peri_lpsys）
pub fn get_clk_peri_lpsys_freq() -> Option<Hertz> {
    match get_clk_peri_lpsys_source() {
        ClkPeriSel::Hrc48 => get_hrc48_freq(),
        ClkPeriSel::Hxt48 => get_hxt48_freq(),
    }
}

/// 获取 BT MAC 时钟频率（MACCLK）
pub fn get_macclk_freq() -> Option<Hertz> {
    let hclk = get_hclk_lpsys_freq()?;
    let div = LPSYS_RCC.cfgr().read().macdiv();

    if div == 0 {
        None
    } else {
        Some(hclk / div as u32)
    }
}

/// 获取 LPSYS systick 时钟源
pub fn get_lpsys_tick_source() -> TickSel {
    LPSYS_RCC.csr().read().sel_tick()
}

/// 获取 LPSYS systick 分频
pub fn get_lpsys_tick_div() -> u8 {
    LPSYS_RCC.cfgr().read().tickdiv()
}

/// 获取 LPSYS systick 参考时钟频率
pub fn get_lpsys_tick_freq() -> Option<Hertz> {
    let base = match get_lpsys_tick_source() {
        // RTC 频率尚未在 HAL 中完整建模，这里先返回 None。
        TickSel::ClkRtc => return None,
        TickSel::Hrc48 => get_hrc48_freq()?,
        TickSel::Hxt48 => get_hxt48_freq()?,
        _ => return None,
    };

    let div = get_lpsys_tick_div();
    if div == 0 {
        Some(base)
    } else {
        Some(base / div as u32)
    }
}

/// 打印 LPSYS 相关时钟，便于调试
pub fn test_print_clocks() {
    info!("LPSYS Clock frequencies:");

    let clocks = [
        ("clk_lpsys", get_clk_lpsys_freq()),
        ("hclk_lpsys", get_hclk_lpsys_freq()),
        ("pclk1_lpsys", get_pclk1_lpsys_freq()),
        ("pclk2_lpsys", get_pclk2_lpsys_freq()),
        ("clk_peri_lpsys", get_clk_peri_lpsys_freq()),
        ("macclk", get_macclk_freq()),
        ("lp_tick", get_lpsys_tick_freq()),
    ];

    for (name, freq) in clocks {
        if let Some(f) = freq {
            let freq_khz = f.0 / 1_000;
            let mhz_part = freq_khz / 1_000;
            let khz_part = freq_khz % 1_000;

            if khz_part == 0 {
                info!("  - {}: {} MHz", name, mhz_part);
            } else if mhz_part == 0 {
                info!("  - {}: {} kHz", name, khz_part);
            } else {
                info!("  - {}: {}.{:03} MHz", name, mhz_part, khz_part);
            }
        } else {
            info!("  - {}: disabled/unknown", name);
        }
    }
}
