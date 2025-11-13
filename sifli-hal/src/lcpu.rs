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
    /// HCPU (High-performance CPU)
    Hcpu = 0,
    /// LCPU (Low-power CPU)
    Lcpu = 1,
    /// BCPU (Bluetooth CPU，部分型号可用)
    Bcpu = 2,
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
    #[cfg(feature = "defmt")]
    defmt::info!("Starting LCPU power-on sequence");

    // 1. 唤醒 LCPU (bf0_lcpu_init.c:165)
    #[cfg(feature = "defmt")]
    defmt::debug!("Step 1: Waking up LCPU");
    wake_core(CoreId::Lcpu)?;

    // 2. 复位并停表 LCPU (bf0_lcpu_init.c:166)
    #[cfg(feature = "defmt")]
    defmt::debug!("Step 2: Resetting and halting LCPU");
    reset_and_halt_lcpu()?;

    // 3. ROM 配置 (bf0_lcpu_init.c:168)
    #[cfg(feature = "defmt")]
    defmt::debug!("Step 3: Configuring ROM parameters");
    lcpu_rom_config()?;

    // 4. 限频检查 (bf0_lcpu_init.c:170-176)
    if !config.skip_frequency_check {
        #[cfg(feature = "defmt")]
        defmt::debug!("Step 4: Checking LCPU frequency (must be ≤ 24MHz during loading)");
        check_lcpu_frequency()?;
    } else {
        #[cfg(feature = "defmt")]
        defmt::warn!("Step 4: Skipping frequency check (as requested by config)");
    }

    // 5. 装载镜像 (bf0_lcpu_init.c:178-182) - 仅 A3 及之前
    let idr = syscfg::SysCfg::read_idr();
    if !idr.revision().is_letter_series() {
        #[cfg(feature = "defmt")]
        defmt::debug!("Step 5: Installing LCPU firmware image (A3/earlier)");

        if let Some(firmware) = config.firmware {
            lcpu_img::install(&idr, firmware)?;
        } else {
            #[cfg(feature = "defmt")]
            defmt::error!("Firmware required for A3 and earlier revisions");
            return Err(LcpuError::FirmwareMissing);
        }
    } else {
        #[cfg(feature = "defmt")]
        defmt::debug!("Step 5: Skipping image install (Letter Series, firmware in ROM)");
    }

    // 6. 配置启动地址 (bf0_lcpu_init.c:184)
    #[cfg(feature = "defmt")]
    defmt::debug!(
        "Step 6: Configuring LCPU start address (0x{:08X})",
        LCPU_CODE_START_ADDR
    );
    lpaon::LpAon::configure_lcpu_start();

    // 7. 安装补丁与 RF 校准 (bf0_lcpu_init.c:185)
    #[cfg(feature = "defmt")]
    defmt::debug!("Step 7: Installing patches and RF calibration");
    install_patch_and_calibrate(config, &idr)?;

    // 8. 释放 LCPU 运行 (bf0_lcpu_init.c:186)
    #[cfg(feature = "defmt")]
    defmt::debug!("Step 8: Releasing LCPU to run");
    release_lcpu()?;

    // 9. 收尾 (bf0_lcpu_init.c:187)
    #[cfg(feature = "defmt")]
    defmt::debug!("Step 9: Cleaning up (cancel LP_ACTIVE request)");
    cancel_lp_active_request()?;

    #[cfg(feature = "defmt")]
    defmt::info!("LCPU power-on sequence completed successfully");

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
    #[cfg(feature = "defmt")]
    defmt::info!("Powering off LCPU");

    reset_and_halt_lcpu()?;

    #[cfg(feature = "defmt")]
    defmt::info!("LCPU powered off successfully");

    Ok(())
}

//=============================================================================
// 内部辅助函数（待实现）
//=============================================================================

/// 唤醒指定核心
///
/// # 待实现
///
/// 对应 SDK: `HAL_HPAON_WakeCore()` in `bf0_hal_hpaon.c:190`
///
/// 需要在 `hpaon` 模块中实现。
fn wake_core(_core_id: CoreId) -> Result<(), LcpuError> {
    // TODO: 实现 HAL_HPAON_WakeCore
    // 参考: SiFli-SDK/drivers/hal/bf0_hal_hpaon.c:190
    //
    // 步骤：
    // 1. 置位 HPSYS_AON->ISSR.HP2LP_REQ
    // 2. 等待 ISSR.LP_ACTIVE 置位
    // 3. 维护引用计数（LB52x）
    todo!("wake_core: HAL_HPAON_WakeCore 实现")
}

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

/// 取消 LP_ACTIVE 请求
///
/// # 待实现
///
/// 对应 SDK: `HAL_HPAON_CANCEL_LP_ACTIVE_REQUEST()` 宏
///
/// 需要在 `hpaon` 模块中实现。
fn cancel_lp_active_request() -> Result<(), LcpuError> {
    // TODO: 实现 HAL_HPAON_CANCEL_LP_ACTIVE_REQUEST
    // 参考: SiFli-SDK/drivers/Include/bf0_hal_aon.h:298-314
    //
    // 步骤：
    // 维护引用计数，归零后清 HPSYS_AON->ISSR.HP2LP_REQ
    todo!("cancel_lp_active_request: HAL_HPAON_CANCEL_LP_ACTIVE_REQUEST 实现")
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
    #[cfg(feature = "defmt")]
    defmt::debug!("Using default ROM configuration");

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
fn install_patch_and_calibrate(config: &LcpuConfig, idr: &Idr) -> Result<(), LcpuError> {
    let revision = idr.revision();

    // 根据版本选择补丁数据
    let patch_data = if revision.is_letter_series() {
        #[cfg(feature = "defmt")]
        defmt::debug!("Using Letter Series patch data");
        config.patch_letter
    } else {
        #[cfg(feature = "defmt")]
        defmt::debug!("Using A3 patch data");
        config.patch_a3
    };

    // 安装补丁（如果提供了数据）
    if let Some(data) = patch_data {
        #[cfg(feature = "defmt")]
        defmt::debug!(
            "Installing patches (record: {} words, code: {} words)",
            data.record.len(),
            data.code.len()
        );

        // TODO: 使用 patch 模块安装补丁
        // 参考: patch::Patch::with_data(record, code).install()
        todo!("install_patch_and_calibrate: 调用 patch 模块安装补丁")
    } else {
        #[cfg(feature = "defmt")]
        defmt::warn!("No patch data provided, skipping patch installation");
    }

    // RF 校准
    if !config.disable_rf_cal {
        #[cfg(feature = "defmt")]
        defmt::debug!("Performing RF calibration");

        // TODO: 调用 RF 校准函数
        // 参考: bt_rf_cal() in SDK
        todo!("install_patch_and_calibrate: 实现 RF 校准")
    } else {
        #[cfg(feature = "defmt")]
        defmt::warn!("RF calibration disabled by config");
    }

    Ok(())
}
