//! HPAON (High-Power Always-On) HAL driver
//! 注意！文档由AI生成！
//!
//! 该模块封装了 HPSYS_AON 里与 LCPU 唤醒相关的少量寄存器操作，
//! 主要对应 SDK 中的：
//! - `HAL_HPAON_WakeCore(CORE_ID_LCPU)`（只保留 HP2LP_REQ/LP_ACTIVE 逻辑）
//! - `HAL_HPAON_CANCEL_LP_ACTIVE_REQUEST()`
//!
//! 当前仅用于 `LcpuActiveGuard`，后续如有需要可以在此处补充更多 HPAON 功能。

use crate::pac::HPSYS_AON;

/// HPAON 驱动命名空间
pub struct Hpaon;

impl Hpaon {
    /// 唤醒 LCPU 所在的 LPSYS/LP 域
    ///
    /// - 置位 `HP2LP_REQ`
    /// - 轮询等待 `LP_ACTIVE` 置位，带简单的循环计数超时
    ///
    /// 返回：
    /// - `true`  表示在给定循环次数内检测到 `LP_ACTIVE`
    /// - `false` 表示超时（未检测到 `LP_ACTIVE`）
    ///
    /// # 对应 SDK
    ///
    /// - `HAL_HPAON_WakeCore(CORE_ID_LCPU)` in `bf0_hal_hpaon.c:190`
    pub fn wake_lcpu_with_timeout(timeout_us: u32) -> bool {
        // 置位 HP2LP_REQ，向 LPSYS/LP 域发出激活请求
        HPSYS_AON.issr().modify(|w| w.set_hp2lp_req(true));

        // SDK 使用两个阶段的 `HAL_Delay_us` + 轮询 LP_ACTIVE。
        // 这里采用更通用的“基于时间的等待 + 超时”模式：
        //
        // - 循环中每次检查 LP_ACTIVE
        // - 使用一个小的固定步长 busy-wait 延时，累计已等待的时间
        // - 超过 timeout_us 仍未就绪则返回 false
        //
        // 这样和 HAL 其他基于时间估算的超时逻辑（如 efuse）风格一致，
        // 也更贴近 SDK 中“以 us 为单位”的等待设计。
        let step_us: u32 = 10;
        let mut waited: u32 = 0;

        loop {
            if HPSYS_AON.issr().read().lp_active() {
                return true;
            }

            if waited >= timeout_us {
                return false;
            }

            crate::cortex_m_blocking_delay_us(step_us);
            waited = waited.saturating_add(step_us);
        }
    }

    /// 取消 LCPU 的 LP_ACTIVE 请求
    ///
    /// 清除 `HP2LP_REQ` 位，对应 SDK 的
    /// `HAL_HPAON_CANCEL_LP_ACTIVE_REQUEST()` 宏。
    pub fn cancel_lp_active_request() {
        HPSYS_AON.issr().modify(|w| w.set_hp2lp_req(false));
    }
}
