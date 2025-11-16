use crate::pac::LPSYS_RCC;
use crate::rcc::ConfigOption;

use super::{ClkPeriSel, ClkSysSel, TickSel};

/// LPSYS RCC 配置
///
/// 与 HPSYS `rcc::Config` 类似，但仅包含 LPSYS 相关的配置项。
/// 所有字段默认使用 [`ConfigOption::Keep`]，即保持当前寄存器值不变。
pub struct LpsysConfig {
    /// LPSYS 主时钟源（SEL_SYS），在使用 WDT 域时仍有效但不会生效
    pub sys_main_src: ConfigOption<ClkSysSel>,
    /// 是否使用 WDT 域作为 LPSYS 时钟（SEL_SYS_LP）
    pub sys_use_wdt: ConfigOption<bool>,
    /// HCLK 分频: hclk_lpsys = clk_lpsys / hclk_div （0 表示不分频）
    pub hclk_div: ConfigOption<u8>,
    /// PCLK1 分频: pclk1_lpsys = hclk_lpsys / 2^pclk1_div
    pub pclk1_div: ConfigOption<u8>,
    /// PCLK2 分频: pclk2_lpsys = hclk_lpsys / 2^pclk2_div
    pub pclk2_div: ConfigOption<u8>,
    /// LPSYS 外设时钟源（SEL_PERI）
    pub peri_src: ConfigOption<ClkPeriSel>,
    /// LPSYS systick 时钟
    pub tick: ConfigOption<LpsysTickConfig>,
}

/// LPSYS systick 时钟配置
pub struct LpsysTickConfig {
    /// systick 参考时钟源（SEL_TICK）
    pub sel: TickSel,
    /// 分频系数：tick = ref / TICKDIV（0 表示不分频）
    pub div: u8,
}

impl LpsysConfig {
    /// 创建一个“保持现状”的配置对象
    pub fn new_keep() -> Self {
        Self {
            sys_main_src: ConfigOption::keep(),
            sys_use_wdt: ConfigOption::keep(),
            hclk_div: ConfigOption::keep(),
            pclk1_div: ConfigOption::keep(),
            pclk2_div: ConfigOption::keep(),
            peri_src: ConfigOption::keep(),
            tick: ConfigOption::keep(),
        }
    }

    /// 应用 LPSYS RCC 配置到硬件
    ///
    /// # Safety
    ///
    /// 修改 LPSYS 时钟会影响 LCPU / BT MAC 等模块。
    /// 调用方必须保证在安全的时机调用，不破坏正在运行的固件。
    pub unsafe fn apply(&self) {
        // 主系统时钟源（SEL_SYS）
        if let ConfigOption::Update(sel) = self.sys_main_src {
            LPSYS_RCC.csr().modify(|w| w.set_sel_sys(sel));
        }

        // 是否使用 WDT 域时钟（SEL_SYS_LP）
        if let ConfigOption::Update(use_wdt) = self.sys_use_wdt {
            LPSYS_RCC.csr().modify(|w| w.set_sel_sys_lp(use_wdt));
        }

        // HCLK / PCLK 分频
        if let ConfigOption::Update(div) = self.hclk_div {
            LPSYS_RCC.cfgr().modify(|w| w.set_hdiv1(div));
        }
        if let ConfigOption::Update(div) = self.pclk1_div {
            LPSYS_RCC.cfgr().modify(|w| w.set_pdiv1(div));
        }
        if let ConfigOption::Update(div) = self.pclk2_div {
            LPSYS_RCC.cfgr().modify(|w| w.set_pdiv2(div));
        }

        // LPSYS 外设时钟源
        if let ConfigOption::Update(sel) = self.peri_src {
            LPSYS_RCC.csr().modify(|w| w.set_sel_peri(sel));
        }

        // systick 时钟
        if let ConfigOption::Update(tick) = &self.tick {
            LPSYS_RCC.csr().modify(|w| w.set_sel_tick(tick.sel));
            LPSYS_RCC.cfgr().modify(|w| w.set_tickdiv(tick.div));
        }
    }
}

impl Default for LpsysConfig {
    fn default() -> Self {
        Self::new_keep()
    }
}
