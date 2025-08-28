use crate::time::Hertz;
use crate::pac::{HPSYS_RCC, HPSYS_AON};

pub use crate::pac::hpsys_rcc::vals::{
    SelSys as ClkSysSel,
    SelUsbc as UsbSel,
    SelTick as TickSel,
    SelPeri as ClkPeriSel,
};

// all clocks:
// clk_sys, clk_peri, clk_peri_div2
// clk_aud_pll, aud_pll_div16
// hxt48, hrc48
// clk_dll1, clk_dll2
// clk_rtc(TODO), clk_wdt(TODO)
// hclk, pclk1, pclk2
// clk_usb
// TODO: lxt32, lrc32, lrc10, clk_mpi1, clk_mpi2

/// clk_sys
pub fn get_clk_sys_freq() -> Option<Hertz> {
    match HPSYS_RCC.csr().read().sel_sys() {
        ClkSysSel::Hrc48 => get_hrc48_freq(),
        ClkSysSel::Hxt48 => get_hxt48_freq(),
        ClkSysSel::Dbl96 => todo!(),
        ClkSysSel::Dll1 => get_clk_dll1_freq(),
    }
}

pub fn get_clk_sys_source() -> ClkSysSel {
    HPSYS_RCC.csr().read().sel_sys()
}

pub fn get_clk_peri_freq() -> Option<Hertz> {
    match HPSYS_RCC.csr().read().sel_peri() {
        ClkPeriSel::Hxt48 => get_hxt48_freq(),
        ClkPeriSel::Hrc48 => get_hrc48_freq(),
    }
}
pub fn get_clk_peri_div2_freq() -> Option<Hertz> {
    match HPSYS_RCC.csr().read().sel_peri() {
        ClkPeriSel::Hxt48 => get_hxt48_freq().map(|f| f / 2u8),
        ClkPeriSel::Hrc48 => get_hrc48_freq().map(|f| f / 2u8),
    }
}

pub fn get_hclk_freq() -> Option<Hertz> {
    let clk_sys = get_clk_sys_freq()?;
    Some(clk_sys / HPSYS_RCC.cfgr().read().hdiv())
}

pub fn get_hclk_div() -> u8 {
    HPSYS_RCC.cfgr().read().hdiv()
}

pub fn get_pclk1_freq() -> Option<Hertz> {
    let hclk = get_hclk_freq()?;
    Some(hclk / (1 << HPSYS_RCC.cfgr().read().pdiv1()) as u32)
}

pub fn get_pclk2_freq() -> Option<Hertz> {
    let hclk = get_hclk_freq()?;
    Some(hclk / (1 << HPSYS_RCC.cfgr().read().pdiv2()) as u32)
}

pub fn get_hxt48_freq() -> Option<Hertz> {
    if HPSYS_AON.acr().read().hxt48_rdy() {
        Some(Hertz(48_000_000))
    } else {
        None
    }
}

pub fn get_hrc48_freq() -> Option<Hertz> {
    if HPSYS_AON.acr().read().hrc48_rdy() {
        Some(Hertz(48_000_000))
    } else {
        None
    }
}

pub fn get_clk_dll1_freq() -> Option<Hertz> {
    let dllcr = HPSYS_RCC.dllcr(0).read();
    if dllcr.en() {
        Some(Hertz(24_000_000 * (dllcr.stg() + 1) as u32 / (dllcr.out_div2_en() as u32 + 1)))
    } else {
        None
    }
}

pub fn get_clk_dll2_freq() -> Option<Hertz> {
    let dllcr = HPSYS_RCC.dllcr(1).read();
    if dllcr.en() {
        Some(Hertz(24_000_000 * (dllcr.stg() + 1) as u32 / (dllcr.out_div2_en() as u32 + 1)))
    } else {
        None
    }
}

pub fn get_clk_aud_pll_freq() -> Option<Hertz> {
    Some(Hertz(49_152_000))
}

pub fn get_clk_aud_pll_div16_freq() -> Option<Hertz> {
    Some(Hertz(49_152_000 / 16))
}

pub fn get_clk_usb_freq() -> Option<Hertz> {
    match HPSYS_RCC.csr().read().sel_usbc() {
        UsbSel::ClkSys => get_clk_sys_freq(),
        UsbSel::Dll2 => get_clk_dll2_freq(),
    }.map(|f| f / HPSYS_RCC.usbcr().read().div())
}

pub fn get_clk_usb_source() -> UsbSel {
    HPSYS_RCC.csr().read().sel_usbc()
}

pub fn get_clk_usb_div() -> u8 {
    HPSYS_RCC.usbcr().read().div()
}

pub fn get_clk_mpi1_freq() -> Option<Hertz> {
    todo!()
}

pub fn get_clk_mpi2_freq() -> Option<Hertz> {
    todo!()
}

// TODO: MPI1 & MPI2
// pub fn get_clk_mpi1_source() -> SelMpi{
//     HPSYS_RCC.csr().read().sel_mpi1()
// }

pub fn get_clk_wdt_freq() -> Option<Hertz> {
    todo!()
}

pub fn get_clk_rtc_freq() -> Option<Hertz> {
    todo!()
}

pub fn test_print_clocks() {
    info!("Clock frequencies:");
    
    let clocks = [
        ("clk_sys", get_clk_sys_freq()),
        ("hclk", get_hclk_freq()),
        ("pclk1", get_pclk1_freq()),
        ("pclk2", get_pclk2_freq()),
        ("clk_peri", get_clk_peri_freq()),
        ("clk_peri_div2", get_clk_peri_div2_freq()),
        ("clk_dll1", get_clk_dll1_freq()),
        ("clk_dll2", get_clk_dll2_freq()),
        ("hxt48", get_hxt48_freq()),
        ("hrc48", get_hrc48_freq()),
        ("clk_usb", get_clk_usb_freq()),
        ("clk_aud_pll", get_clk_aud_pll_freq()),
        ("clk_aud_pll_div16", get_clk_aud_pll_div16_freq()),
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
            info!("  - {}: {}.{:03} MHz",
                  name,
                  mhz_part,
                  khz_part
            );
        }
    } else {
        info!("  - {}: disabled", name);
    }
}
}
