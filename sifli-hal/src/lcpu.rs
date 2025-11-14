//! LCPU 启动和电源管理模块
//!
//! ## 芯片版本差异
//!
//! ### A3 及更早版本
//! - 需要通过 HCPU 将 LCPU 固件复制到 LPSYS RAM
//! - 使用 `lcpu_patch.c` 格式的补丁数据
//! - 补丁记录区 + 补丁代码区
//!
//! ### Letter Series (A4/B4)
//! - LCPU 固件已烧录在 ROM，无需复制
//! - 使用 `lcpu_patch_rev_b.c` 格式的补丁数据
//! - Header (12 bytes) + 补丁代码区
//!
//! ## 使用示例
//!
//! ```no_run
//! use sifli_hal::lcpu::{self, LcpuConfig, PatchData};
//!
//! # fn example() -> Result<(), sifli_hal::lcpu::LcpuError> {
//! // 准备配置
//! let config = LcpuConfig {
//!     firmware: Some(&LCPU_FIRMWARE),  // A3 固件镜像
//!     patch_a3: Some(PatchData {
//!         record: &PATCH_RECORD_A3,
//!         code: &PATCH_CODE_A3,
//!     }),
//!     patch_letter: Some(PatchData {
//!         record: &PATCH_RECORD_LETTER,
//!         code: &PATCH_CODE_LETTER,
//!     }),
//!     skip_frequency_check: false,
//!     disable_rf_cal: false,
//! };
//!
//! // 启动 LCPU
//! lcpu::power_on(&config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 参考资料
//!
//! - SDK 实现: `SiFli-SDK/drivers/cmsis/sf32lb52x/bf0_lcpu_init.c`
//! - 详细流程: `docs/dev_notes/lcpu_startup_flow.md`
//! - 镜像安装: [`lcpu_img`] 模块
//! - 启动地址配置: [`lpaon`] 模块
//! - 补丁管理: [`patch`] 模块

// 重导出相关模块
pub use crate::lcpu_img::{self, LCPU_CODE_START_ADDR, LPSYS_RAM_BASE, LPSYS_RAM_SIZE};

use crate::lpaon;
use crate::patch;
use crate::syscfg::{self, Idr};

//=============================================================================
// 配置结构
//=============================================================================

/// LCPU 启动配置
///
/// 提供 LCPU 启动所需的所有参数，包括固件镜像、补丁数据和控制选项。
#[derive(Debug, Clone, Copy)]
pub struct LcpuConfig {
    /// LCPU 固件镜像（u32 数组）
    ///
    /// - **A3 及之前版本**：必须提供，用于复制到 LPSYS RAM
    /// - **Letter Series**：可选，固件已在 ROM 中
    pub firmware: Option<&'static [u32]>,

    /// A3 及更早版本的补丁数据
    ///
    /// 格式：record + code (来自 `lcpu_patch.c`)
    pub patch_a3: Option<PatchData>,

    /// Letter Series (A4/B4) 的补丁数据
    ///
    /// 格式：header + code (来自 `lcpu_patch_rev_b.c`)
    pub patch_letter: Option<PatchData>,

    /// 是否跳过频率检查
    ///
    /// 装载镜像期间，LPSYS HCLK 不应超过 24MHz。
    /// 设置为 `true` 可跳过此检查（⚠️ 谨慎使用）。
    pub skip_frequency_check: bool,

    /// 是否禁用 RF 校准
    ///
    /// RF 校准通常在补丁安装后执行。设置为 `true` 可跳过。
    pub disable_rf_cal: bool,
}

impl LcpuConfig {
    /// 创建默认配置
    ///
    /// 默认不提供固件和补丁数据，需要手动设置。
    pub const fn new() -> Self {
        Self {
            firmware: None,
            patch_a3: None,
            patch_letter: None,
            skip_frequency_check: false,
            disable_rf_cal: false,
        }
    }

    /// 设置 A3 固件镜像
    pub const fn with_firmware(mut self, firmware: &'static [u32]) -> Self {
        self.firmware = Some(firmware);
        self
    }

    /// 设置 A3 补丁数据
    pub const fn with_patch_a3(mut self, patch: PatchData) -> Self {
        self.patch_a3 = Some(patch);
        self
    }

    /// 设置 Letter Series 补丁数据
    pub const fn with_patch_letter(mut self, patch: PatchData) -> Self {
        self.patch_letter = Some(patch);
        self
    }

    /// 跳过频率检查
    pub const fn skip_frequency_check(mut self) -> Self {
        self.skip_frequency_check = true;
        self
    }

    /// 禁用 RF 校准
    pub const fn disable_rf_cal(mut self) -> Self {
        self.disable_rf_cal = true;
        self
    }
}

impl Default for LcpuConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 补丁数据
///
/// 包含补丁记录和补丁代码的 u32 数组。
#[derive(Debug, Clone, Copy)]
pub struct PatchData {
    /// 补丁记录数组
    pub record: &'static [u32],
    /// 补丁代码数组
    pub code: &'static [u32],
}

//=============================================================================
// 错误类型
//=============================================================================

/// LCPU 启动错误
#[derive(Debug)]
pub enum LcpuError {
    /// 镜像安装错误
    ImageInstall(lcpu_img::Error),

    /// 补丁安装错误
    PatchInstall(patch::Error),

    /// 缺少固件
    ///
    /// A3 及之前版本必须提供固件镜像。
    FirmwareMissing,

    /// 频率检查失败
    ///
    /// 装载期间 LPSYS HCLK 超过 24MHz 限制。
    FrequencyTooHigh {
        /// 实际频率 (Hz)
        actual_hz: u32,
        /// 最大允许频率 (Hz)
        max_hz: u32,
    },

    /// HPAON 操作错误
    HpaonError,

    /// RCC 操作错误
    RccError,

    /// ROM 配置错误
    RomConfigError,

    /// 引用计数溢出
    ///
    /// 超过最大允许的 LCPU 引用计数（20）。
    RefCountOverflow,

    /// 唤醒核心超时
    ///
    /// 等待 LP_ACTIVE 信号超时。
    WakeCoreTimeout,
}

impl From<lcpu_img::Error> for LcpuError {
    fn from(err: lcpu_img::Error) -> Self {
        Self::ImageInstall(err)
    }
}

impl From<patch::Error> for LcpuError {
    fn from(err: patch::Error) -> Self {
        Self::PatchInstall(err)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for LcpuError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Self::ImageInstall(err) => {
                defmt::write!(fmt, "ImageInstall({:?})", err);
            }
            Self::PatchInstall(err) => {
                defmt::write!(fmt, "PatchInstall({:?})", err);
            }
            Self::FirmwareMissing => {
                defmt::write!(fmt, "FirmwareMissing");
            }
            Self::FrequencyTooHigh { actual_hz, max_hz } => {
                defmt::write!(
                    fmt,
                    "FrequencyTooHigh {{ actual_hz: {=u32}, max_hz: {=u32} }}",
                    actual_hz,
                    max_hz
                );
            }
            Self::HpaonError => {
                defmt::write!(fmt, "HpaonError");
            }
            Self::RccError => {
                defmt::write!(fmt, "RccError");
            }
            Self::RomConfigError => {
                defmt::write!(fmt, "RomConfigError");
            }
            Self::RefCountOverflow => {
                defmt::write!(fmt, "RefCountOverflow");
            }
            Self::WakeCoreTimeout => {
                defmt::write!(fmt, "WakeCoreTimeout");
            }
        }
    }
}

//=============================================================================
// 核心 ID 定义
//=============================================================================

/// 核心 ID（用于多核唤醒操作）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CoreId {
    Default = 0,
    /// HCPU (High-performance CPU)
    Hcpu = 1,
    /// LCPU (Low-power CPU)
    Lcpu = 2,
    /// ACPU (58x)
    Acpu = 3,
}

//=============================================================================
// LCPU 活跃状态守卫与引用计数
//=============================================================================

use core::sync::atomic::{AtomicU8, Ordering};

/// LCPU 唤醒引用计数器
///
/// 追踪有多少个模块正在使用 LCPU。
/// 仅当计数归零时才真正清除 HP2LP_REQ。
static LCPU_WAKEUP_REF_CNT: AtomicU8 = AtomicU8::new(0);

/// 引用计数最大值（来自 SDK）
const MAX_REF_COUNT: u8 = 20;

/// LCPU 活跃状态守卫
///
/// 采用 RAII 模式管理 LCPU 的唤醒与睡眠。
///
/// - 创建时：执行 wake_core 逻辑（置位 HP2LP_REQ，等待 LP_ACTIVE）
/// - Drop 时：执行 cancel_lp_active_request 逻辑（清除 HP2LP_REQ）
///
/// 内部维护引用计数，支持多个模块同时持有 Guard。
///
/// # Example
///
/// ```no_run
/// use sifli_hal::lcpu::LcpuActiveGuard;
///
/// # fn example() -> Result<(), sifli_hal::lcpu::LcpuError> {
/// // 创建 Guard，LCPU 被唤醒
/// let _guard = LcpuActiveGuard::new()?;
///
/// // 使用 LCPU...
///
/// // Guard Drop 时自动取消 LP_ACTIVE 请求
/// # Ok(())
/// # }
/// ```
///
/// # 对应 SDK 函数
///
/// - 创建：`HAL_HPAON_WakeCore()` in `bf0_hal_hpaon.c:190`
/// - Drop：`HAL_HPAON_CANCEL_LP_ACTIVE_REQUEST()` in `bf0_hal_aon.h:298`
pub struct LcpuActiveGuard {
    _private: (),
}

impl LcpuActiveGuard {
    /// 创建 LCPU 活跃状态守卫，唤醒 LCPU
    ///
    /// 此函数会：
    /// 1. 原子地增加引用计数
    /// 2. 如果是首次唤醒（0→1），置位 HP2LP_REQ 并等待 LP_ACTIVE
    ///
    /// # Errors
    ///
    /// - [`LcpuError::RefCountOverflow`][]: 引用计数超过最大值（20）
    /// - [`LcpuError::WakeCoreTimeout`][]: 等待 LP_ACTIVE 超时
    ///
    /// # 对应 SDK 函数
    ///
    /// - `HAL_HPAON_WakeCore(CORE_ID_LCPU)` in `bf0_hal_hpaon.c:190`
    pub fn new() -> Result<Self, LcpuError> {
        // 1. 原子增加引用计数
        let prev_count = critical_section::with(|_| {
            let count = LCPU_WAKEUP_REF_CNT.load(Ordering::Relaxed);
            if count >= MAX_REF_COUNT {
                return Err(LcpuError::RefCountOverflow);
            }
            LCPU_WAKEUP_REF_CNT.store(count + 1, Ordering::Relaxed);
            Ok(count)
        })?;

        debug!(
            "LCPU wake reference count: {} -> {}",
            prev_count,
            prev_count + 1
        );

        // 2. 仅在首次唤醒时执行硬件操作（0→1）
        if prev_count == 0 {
            debug!("First wake, setting HP2LP_REQ and waiting for LP_ACTIVE");

            let hpsys_aon = crate::pac::HPSYS_AON;

            // 置位 HP2LP_REQ (bf0_hal_hpaon.c:206)
            hpsys_aon.issr().modify(|w| w.set_hp2lp_req(true));

            // 等待 LP_ACTIVE 置位 (bf0_hal_hpaon.c:212-217)
            // 超时时间：假设 48MHz 时钟，1M 周期约 20ms
            const TIMEOUT_CYCLES: u32 = 1_000_000;
            let mut timeout = TIMEOUT_CYCLES;

            while !hpsys_aon.issr().read().lp_active() {
                timeout = timeout.saturating_sub(1);
                if timeout == 0 {
                    error!("Timeout waiting for LP_ACTIVE, rolling back ref count");

                    // 超时：回滚引用计数
                    critical_section::with(|_| {
                        LCPU_WAKEUP_REF_CNT.store(0, Ordering::Relaxed);
                    });

                    return Err(LcpuError::WakeCoreTimeout);
                }
            }

            debug!("LP_ACTIVE set, LCPU is now awake");
        } else {
            debug!("LCPU already awake, reusing existing wake state");
        }

        Ok(Self { _private: () })
    }
}

impl Drop for LcpuActiveGuard {
    /// 析构时取消 LP_ACTIVE 请求
    ///
    /// 此函数会：
    /// 1. 原子地减少引用计数
    /// 2. 如果是最后一个释放（1→0），清除 HP2LP_REQ
    ///
    /// # 对应 SDK 函数
    ///
    /// - `HAL_HPAON_CANCEL_LP_ACTIVE_REQUEST()` 宏 in `bf0_hal_aon.h:298`
    fn drop(&mut self) {
        // 原子减少引用计数，判断是否是最后一个
        let is_last = critical_section::with(|_| {
            let count = LCPU_WAKEUP_REF_CNT.load(Ordering::Relaxed);
            assert!(count >= 1, "LCPU wake reference count underflow");
            LCPU_WAKEUP_REF_CNT.store(count - 1, Ordering::Relaxed);
            count == 1
        });

        let new_count = LCPU_WAKEUP_REF_CNT.load(Ordering::Relaxed);
        debug!("LCPU wake reference count decreased to {}", new_count);

        // 仅在最后一个 Guard 释放时清除 HP2LP_REQ（1→0）
        if is_last {
            debug!("Last guard dropped, clearing HP2LP_REQ");

            let hpsys_aon = crate::pac::HPSYS_AON;

            // 清除 HP2LP_REQ (bf0_hal_aon.h:309)
            hpsys_aon.issr().modify(|w| w.set_hp2lp_req(false));

            debug!("HP2LP_REQ cleared, LPSYS can now sleep");
        } else {
            debug!("Other guards still active, keeping HP2LP_REQ set");
        }
    }
}

//=============================================================================
// 启动流程主函数
//=============================================================================

/// 启动 LCPU
///
/// 执行完整的 LCPU 启动流程，包括唤醒、镜像装载、补丁安装、启动配置等。
///
/// # Arguments
///
/// - `config`: LCPU 启动配置，包含固件、补丁数据等
///
/// # 流程说明
///
/// 本函数按照以下顺序执行 LCPU 启动（参考 `bf0_lcpu_init.c:163`）：
///
/// 1. 唤醒 LCPU
/// 2. 复位并停表
/// 3. ROM 参数配置
/// 4. 限频检查（装载期间 ≤ 24MHz）
/// 5. 装载镜像（仅 A3 及之前）
/// 6. 配置启动地址
/// 7. 安装补丁与 RF 校准
/// 8. 释放 LCPU 运行
/// 9. 收尾处理
///
/// # Errors
///
/// - [`LcpuError::FirmwareMissing`][]: A3 版本未提供固件
/// - [`LcpuError::ImageInstall`][]: 镜像安装失败
/// - [`LcpuError::PatchInstall`][]: 补丁安装失败
/// - [`LcpuError::FrequencyTooHigh`][]: 频率检查失败
/// - [`LcpuError::HpaonError`][]: HPAON 操作失败
/// - [`LcpuError::RccError`][]: RCC 操作失败
///
/// # Example
///
/// ```no_run
/// use sifli_hal::lcpu::{self, LcpuConfig, PatchData};
///
/// # fn example() -> Result<(), sifli_hal::lcpu::LcpuError> {
/// let config = LcpuConfig::new()
///     .with_firmware(&LCPU_FIRMWARE)
///     .with_patch_a3(PatchData {
///         record: &PATCH_RECORD_A3,
///         code: &PATCH_CODE_A3,
///     });
///
/// lcpu::power_on(&config)?;
/// # Ok(())
/// # }
/// ```
///
/// # 对应 SDK 函数
///
/// - SDK: `lcpu_power_on()` in `bf0_lcpu_init.c:163`
pub fn power_on(config: &LcpuConfig) -> Result<(), LcpuError> {
    info!("Starting LCPU power-on sequence");

    // 1. 唤醒 LCPU (bf0_lcpu_init.c:165)
    // 使用 LcpuActiveGuard 自动管理唤醒与取消
    debug!("Step 1: Waking up LCPU");
    let _guard = LcpuActiveGuard::new()?;

    // 2. 复位并停表 LCPU (bf0_lcpu_init.c:166)
    debug!("Step 2: Resetting and halting LCPU");
    reset_and_halt_lcpu()?;

    // 3. ROM 配置 (bf0_lcpu_init.c:168)
    debug!("Step 3: Configuring ROM parameters");
    lcpu_rom_config()?;

    // 4. 限频检查 (bf0_lcpu_init.c:170-176)
    if !config.skip_frequency_check {
        debug!("Step 4: Checking LCPU frequency (must be ≤ 24MHz during loading)");
        check_lcpu_frequency()?;
    } else {
        warn!("Step 4: Skipping frequency check (as requested by config)");
    }

    // 5. 装载镜像 (bf0_lcpu_init.c:178-182) - 仅 A3 及之前
    let idr = syscfg::read_idr();
    if !idr.revision().is_letter_series() {
        debug!("Step 5: Installing LCPU firmware image (A3/earlier)");

        if let Some(firmware) = config.firmware {
            lcpu_img::install(&idr, firmware)?;
        } else {
            error!("Firmware required for A3 and earlier revisions");
            return Err(LcpuError::FirmwareMissing);
        }
    } else {
        debug!("Step 5: Skipping image install (Letter Series, firmware in ROM)");
    }

    // 6. 配置启动地址 (bf0_lcpu_init.c:184)
    debug!(
        "Step 6: Configuring LCPU start address (0x{:08X})",
        LCPU_CODE_START_ADDR
    );
    lpaon::LpAon::configure_lcpu_start();

    // 7. 安装补丁与 RF 校准 (bf0_lcpu_init.c:185)
    debug!("Step 7: Installing patches and RF calibration");
    install_patch_and_calibrate(config, &idr)?;

    // 8. 释放 LCPU 运行 (bf0_lcpu_init.c:186)
    debug!("Step 8: Releasing LCPU to run");
    release_lcpu()?;

    // 9. 收尾 (bf0_lcpu_init.c:187)
    // Guard 会在函数结束时自动 Drop，执行 cancel_lp_active_request
    debug!("Step 9: Guard will auto-cancel LP_ACTIVE request on drop");

    info!("LCPU power-on sequence completed successfully");

    Ok(())
}

/// 关闭 LCPU
///
/// 复位并停表 LCPU，将其置于停止状态。
///
/// # 对应 SDK 函数
///
/// - SDK: `lcpu_power_off()` in `bf0_lcpu_init.c:196`
pub fn power_off() -> Result<(), LcpuError> {
    info!("Powering off LCPU");

    reset_and_halt_lcpu()?;

    info!("LCPU powered off successfully");

    Ok(())
}

//=============================================================================
// 内部辅助函数（待实现）
//=============================================================================

/// 复位并停表 LCPU
///
/// # 待实现
///
/// 对应 SDK: `HAL_RCC_Reset_and_Halt_LCPU()` in `bf0_hal_rcc.c:1989`
///
/// 需要在 `rcc` 模块中实现。
fn reset_and_halt_lcpu() -> Result<(), LcpuError> {
    // TODO: 实现 HAL_RCC_Reset_and_Halt_LCPU
    // 参考: SiFli-SDK/drivers/hal/bf0_hal_rcc.c:1989
    //
    // 步骤：
    // 1. 置位 LPSYS_AON->PMR.CPUWAIT
    // 2. 复位 LCPU 与相关模块
    // 3. 若 LPSYS 睡眠，置位 SLP_CTRL.WKUP_REQ
    // 4. 清除复位位，保持 CPUWAIT=1
    todo!("reset_and_halt_lcpu: HAL_RCC_Reset_and_Halt_LCPU 实现")
}

/// 释放 LCPU 运行
///
/// # 待实现
///
/// 对应 SDK: `HAL_RCC_ReleaseLCPU()` in `bf0_hal_rcc.c:1962`
///
/// 需要在 `rcc` 模块中实现。
fn release_lcpu() -> Result<(), LcpuError> {
    // TODO: 实现 HAL_RCC_ReleaseLCPU
    // 参考: SiFli-SDK/drivers/hal/bf0_hal_rcc.c:1962
    //
    // 步骤：
    // 清除 LPSYS_AON->PMR.CPUWAIT
    todo!("release_lcpu: HAL_RCC_ReleaseLCPU 实现")
}

/// LCPU ROM 配置
///
/// # 待实现
///
/// 对应 SDK: `lcpu_rom_config()` in `bf0_lcpu_init.c:119`
///
/// 使用默认配置，可由用户通过弱符号覆盖。
fn lcpu_rom_config() -> Result<(), LcpuError> {
    // TODO: 实现 lcpu_rom_config
    // 参考: SiFli-SDK/drivers/cmsis/sf32lb52x/bf0_lcpu_init.c:89
    //
    // 典型配置：
    // - LXT 是否使能
    // - 看门狗状态/超时/时钟
    // - BT RC 校准开关
    // - A4+: HCPU→LCPU TX 队列基址
    debug!("Using default ROM configuration");

    todo!("lcpu_rom_config: 实现默认 ROM 配置")
}

/// 检查 LCPU 频率是否满足装载要求
///
/// # 待实现
///
/// 装载镜像期间，LPSYS HCLK 不应超过 24MHz。
///
/// 对应 SDK: `bf0_lcpu_init.c:170-176`
fn check_lcpu_frequency() -> Result<(), LcpuError> {
    // TODO: 实现频率检查
    // 参考: SiFli-SDK/drivers/cmsis/sf32lb52x/bf0_lcpu_init.c:170
    //
    // 步骤：
    // 1. 调用 HAL_RCC_GetHCLKFreq(CORE_ID_LCPU)
    // 2. 若 > 24MHz，临时设置分频
    const MAX_LOAD_FREQ_HZ: u32 = 24_000_000;

    // let actual_hz = rcc::get_lcpu_hclk_freq();
    // if actual_hz > MAX_LOAD_FREQ_HZ {
    //     return Err(LcpuError::FrequencyTooHigh {
    //         actual_hz,
    //         max_hz: MAX_LOAD_FREQ_HZ,
    //     });
    // }

    todo!("check_lcpu_frequency: 实现 LCPU 频率检查")
}

/// 安装补丁与 RF 校准
///
/// 根据芯片版本选择对应的补丁数据并安装，然后执行 RF 校准。
///
/// # 参数
///
/// - `config`: LCPU 配置，包含补丁数据
/// - `idr`: 芯片识别信息（通过 [`syscfg::read_idr()`] 获取）
///
/// # 错误
///
/// - [`LcpuError::PatchInstall`][]: 补丁安装失败
fn install_patch_and_calibrate(config: &LcpuConfig, idr: &Idr) -> Result<(), LcpuError> {
    let revision = idr.revision();

    // 根据版本选择补丁数据
    let patch_data = if revision.is_letter_series() {
        debug!("Using Letter Series patch data");
        config.patch_letter
    } else {
        debug!("Using A3 patch data");
        config.patch_a3
    };

    // 安装补丁（如果提供了数据）
    if let Some(data) = patch_data {
        debug!(
            "Installing patches (record: {} words, code: {} words)",
            data.record.len(),
            data.code.len()
        );

        // 使用 patch 模块安装补丁
        patch::install(idr, data.record, data.code)?;
    } else {
        warn!("No patch data provided, skipping patch installation");
    }

    // RF 校准
    if !config.disable_rf_cal {
        debug!("Performing RF calibration");

        // TODO: 调用 RF 校准函数
        // 参考: bt_rf_cal() in SDK
        todo!("install_patch_and_calibrate: 实现 RF 校准")
    } else {
        warn!("RF calibration disabled by config");
    }

    Ok(())
}
