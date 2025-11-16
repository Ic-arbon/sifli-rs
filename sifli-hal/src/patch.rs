//! LCPU 补丁安装模块
//! 注意！文档由AI生成！
//!
//! SF32LB52x 的 LCPU (Low-power CPU) 需要在启动时安装固件补丁和 RF 校准数据。
//! 根据芯片版本不同,补丁格式和安装方式有所差异。
//!
//! ## 使用示例
//!
//! ```no_run
//! use sifli_hal::{patch, syscfg};
//!
//! // 读取芯片版本信息
//! let idr = syscfg::read_idr();
//!
//! // 自动识别版本并安装补丁
//! patch::install(&idr, &PATCH_LIST, &PATCH_BIN)?;
//!
//! ```
//!
//! ## 补丁格式
//!
//! ### A3 及更早版本
//!
//! - **补丁记录区** (256 bytes): 位于 `0x20407F00`
//! - **补丁代码区** (约 8KB): 位于 `0x20406000`
//! - **格式**: `PATCH_TAG (u32) | size (u32) | entries[] | code[]`
//! - **数据来源**: `lcpu_patch.c` (通过 `carray2rs` 工具转换)
//!
//! 安装步骤:
//! 1. 复制补丁记录数组到 A3 补丁记录区 (`0x20407F00`, 对应 SDK `LCPU_PATCH_RECORD_ADDR`)
//! 2. 清零补丁代码区 (`0x20406000` 起，大小 8KB)
//! 3. 复制补丁代码数组到 A3 补丁代码区 (`0x20406000`)
//!
//! ### Letter Series (A4/B4)
//!
//! - **Header** (12 bytes): 位于 `0x20405000`
//! - **补丁代码区** (约 12KB): 位于 `0x2040500C`
//! - **格式**: `Header { magic, entry_count, code_addr } | code[]`
//! - **数据来源**: `lcpu_patch_rev_b.c` (通过 `carray2rs` 工具转换)
//!
//! 安装步骤:
//! 1. 写入 Header (12 bytes) 到补丁缓冲区起始 (`0x20405000`)
//! 2. 清零补丁代码区 (`0x2040500C` 起，大小约 12KB - 12 bytes)
//! 3. 复制补丁代码数组到 Letter Series 补丁代码区 (`0x2040500C`)
//!
//! ## Header 格式 (Letter Series)
//!
//! ```text
//! Offset | Size | Field        | Value
//! -------|------|--------------|---------------------------
//! 0x00   | 4    | magic        | 0x48434150 ("PACH" 小端)
//! 0x04   | 4    | entry_count  | 补丁条目数量 (通常为 7)
//! 0x08   | 4    | code_addr    | 补丁代码地址 + 1 (Thumb bit)
//! ```
//!
//! ## 内存布局
//!
//! ### A3 及更早版本
//!
//! ```text
//! 0x20406000  ┌─────────────────────────┐
//!             │  补丁代码区              │
//!             │  (约 7.75KB)             │
//! 0x20407F00  ├─────────────────────────┤
//!             │  补丁记录区 (256 bytes)  │
//! 0x20408000  └─────────────────────────┘
//! ```
//!
//! ### Letter Series
//!
//! ```text
//! 0x20405000  ┌─────────────────────────┐
//!             │  Header (12 bytes)      │
//! 0x2040500C  ├─────────────────────────┤
//!             │  补丁代码区              │
//!             │  (约 12KB - 12 bytes)   │
//! 0x20408000  └─────────────────────────┘
//! ```
//!
//! ## 参考资料
//!
//! - SDK 实现 (A3): `SiFli-SDK/drivers/cmsis/sf32lb52x/lcpu_patch.c`
//! - SDK 实现 (Letter): `SiFli-SDK/drivers/cmsis/sf32lb52x/lcpu_patch_rev_b.c`
//! - 内存映射: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h`
//! - HAL 补丁函数: `SiFli-SDK/drivers/hal/bf0_hal_patch.c`

use crate::syscfg::Idr;

//=============================================================================
// 内存布局类型
//=============================================================================

/// A3 及更早版本的 LCPU 补丁内存布局（HCPU 视角）
///
/// 地址和大小与 SDK `mem_map.h` 保持一致，仅在本模块内部使用。
#[derive(Debug, Clone, Copy)]
struct A3PatchLayout;

impl A3PatchLayout {
    /// 补丁代码起始地址
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h:328`
    const CODE_START: usize = 0x2040_6000;

    /// 补丁记录区地址
    ///
    /// 位于补丁区末尾 256 bytes (0x20406000 + 0x2000 - 0x100 = 0x20407F00)
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h:331` (`LCPU_PATCH_RECORD_ADDR`)
    const RECORD_ADDR: usize = 0x2040_7F00;

    /// 补丁总大小
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h:300`
    const TOTAL_SIZE: usize = 8 * 1024;
}

/// Letter Series (A4+/Rev B) 的 LCPU 补丁内存布局（HCPU 视角）
///
/// 地址和大小与 SDK `mem_map.h` 保持一致，仅在本模块内部使用。
#[derive(Debug, Clone, Copy)]
struct LetterPatchLayout;

impl LetterPatchLayout {
    /// 补丁缓冲区起始地址
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h:334`
    const BUF_START: usize = 0x2040_5000;

    /// 补丁代码起始地址
    ///
    /// 位于 Header 之后 (0x20405000 + 12 = 0x2040500C)
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h:335`
    const CODE_START: usize = 0x2040_500C;

    /// 补丁缓冲区大小
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h:337`
    const BUF_SIZE: usize = 0x3000; // 12KB

    /// 补丁代码大小
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/mem_map.h:338`
    const CODE_SIZE: usize = 0x2FF4; // 12KB - 12 bytes

    /// Letter Series 补丁 Header 魔数
    ///
    /// Reference: `SiFli-SDK/drivers/cmsis/sf32lb52x/lcpu_patch_rev_b.c:60`
    const MAGIC: u32 = 0x4843_4150; // "PACH" (little-endian)

    /// Header 中的 entry_count 固定值
    const ENTRY_COUNT: u32 = 7;
}

// ===== 通用常量 =====

/// 补丁标识 (Magic Number)
///
/// Reference: `SiFli-SDK/drivers/Include/bf0_hal_patch.h:83`
#[allow(dead_code)]
const PATCH_TAG: u32 = 0x5054_4348; // "PTCH" (big-endian in memory)

//=============================================================================
// 错误类型
//=============================================================================

/// 补丁安装错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// 补丁记录为空
    EmptyRecord,

    /// 补丁代码为空
    EmptyCode,

    /// 补丁代码超出可用空间
    CodeTooLarge {
        /// 实际大小 (bytes)
        size_bytes: usize,
        /// 最大允许大小 (bytes)
        max_bytes: usize,
    },

    /// 无效的芯片版本
    InvalidRevision {
        /// 版本 ID
        revid: u8,
    },
}

//=============================================================================
// 核心 API
//=============================================================================

/// 安装 LCPU 补丁
///
/// 自动根据芯片版本选择正确的安装方式:
/// - A3 及更早版本: 使用 [`install_a3`]
/// - Letter Series (A4+): 使用 [`install_letter`]
///
/// # Arguments
///
/// - `idr`: 芯片识别信息 (通过 [`crate::syscfg::read_idr()`] 获取)
/// - `list`: 补丁记录/条目列表数组 (u32 words)，对应 `g_lcpu_patch_list`
/// - `bin`:  补丁代码数组 (u32 words)，对应 `g_lcpu_patch_bin`
///
/// # Errors
///
/// - [`Error::EmptyRecord`][]: 补丁记录为空
/// - [`Error::EmptyCode`][]: 补丁代码为空
/// - [`Error::CodeTooLarge`][]: 补丁代码超出可用空间
/// - [`Error::InvalidRevision`][]: 无效的芯片版本
///
/// # Example
///
/// ```no_run
/// use sifli_hal::{patch, syscfg};
///
/// let idr = syscfg::read_idr();
/// patch::install(&idr, &PATCH_LIST, &PATCH_BIN)?;
/// ```
///
/// # 对应 SDK 函数
///
/// - A3: `lcpu_patch_install()` in `lcpu_patch.c:537`
/// - Letter: `lcpu_patch_install_rev_b()` in `lcpu_patch_rev_b.c:58`
pub fn install(idr: &Idr, list: &[u32], bin: &[u32]) -> Result<(), Error> {
    // 参数检查
    if list.is_empty() {
        return Err(Error::EmptyRecord);
    }
    if bin.is_empty() {
        return Err(Error::EmptyCode);
    }

    let revision = idr.revision();
    if !revision.is_valid() {
        return Err(Error::InvalidRevision { revid: idr.revid });
    }

    // 根据版本选择安装方式
    if revision.is_letter_series() {
        install_letter(list, bin)
    } else {
        install_a3(list, bin)
    }
}

/// 安装 A3 格式补丁 (内部函数)
///
/// # 安装步骤
///
/// 1. 复制补丁记录数组到 A3 补丁记录区
/// 2. 清零补丁代码区 (8KB)
/// 3. 复制补丁代码到 A3 补丁代码区
///
/// # Arguments
///
/// - `list`: 补丁记录/条目列表数组 (u32 words)，对应 `g_lcpu_patch_list`
/// - `bin`:  补丁代码数组 (u32 words)，对应 `g_lcpu_patch_bin`
///
/// # Errors
///
/// - [`Error::CodeTooLarge`]: 补丁代码超出 8KB 限制
///
/// # 对应 SDK 函数
///
/// - SDK: `lcpu_patch_install()` in `lcpu_patch.c:537-549`
fn install_a3(list: &[u32], bin: &[u32]) -> Result<(), Error> {
    let code_size = core::mem::size_of_val(bin);
    if code_size > A3PatchLayout::TOTAL_SIZE {
        return Err(Error::CodeTooLarge {
            size_bytes: code_size,
            max_bytes: A3PatchLayout::TOTAL_SIZE,
        });
    }

    debug!(
        "Installing A3 patch: record={} words, code={} bytes",
        list.len(),
        code_size
    );

    unsafe {
        // 1. 复制补丁记录 (entry list)
        let record_dst = A3PatchLayout::RECORD_ADDR as *mut u32;
        core::ptr::copy_nonoverlapping(list.as_ptr(), record_dst, list.len());

        // 2. 清零补丁代码区
        let code_dst = A3PatchLayout::CODE_START as *mut u8;
        core::ptr::write_bytes(code_dst, 0, A3PatchLayout::TOTAL_SIZE);

        // 3. 复制补丁代码
        let code_dst = A3PatchLayout::CODE_START as *mut u32;
        core::ptr::copy_nonoverlapping(bin.as_ptr(), code_dst, bin.len());
    }

    debug!("A3 patch installed successfully");
    Ok(())
}

/// 安装 Letter Series 格式补丁 (内部函数)
///
/// # 安装步骤
///
/// 1. 写入 Header (12 bytes) 到 Letter Series 补丁缓冲区起始地址
///    - Magic: `0x48434150` ("PACH")
///    - Entry count: `7` (固定值,参考 SDK)
///    - Code address: 补丁代码地址 + 1 (Thumb bit)
/// 2. 清零补丁代码区 (约 12KB)
/// 3. 复制补丁代码到 Letter Series 补丁代码区
///
/// # Arguments
///
/// - `list`: 补丁记录/条目列表数组 (u32 words, 未使用但保持接口一致性)
/// - `bin`:  补丁代码数组 (u32 words)
///
/// # Errors
///
/// - [`Error::CodeTooLarge`]: 补丁代码超出 12KB - 12 bytes 限制
///
/// # 对应 SDK 函数
///
/// - SDK: `lcpu_patch_install_rev_b()` in `lcpu_patch_rev_b.c:58-73`
fn install_letter(_list: &[u32], bin: &[u32]) -> Result<(), Error> {
    let code_size = core::mem::size_of_val(bin);
    if code_size > LetterPatchLayout::CODE_SIZE {
        return Err(Error::CodeTooLarge {
            size_bytes: code_size,
            max_bytes: LetterPatchLayout::CODE_SIZE,
        });
    }

    debug!(
        "Installing Letter Series patch: code={} bytes",
        code_size
    );

    unsafe {
        // 1. 写入 Header (12 bytes)
        // Reference: lcpu_patch_rev_b.c:60-66
        let header = [
            LetterPatchLayout::MAGIC,                          // magic: "PACH"
            LetterPatchLayout::ENTRY_COUNT,                    // entry_count (固定值)
            LetterPatchLayout::CODE_START as u32 + 1,          // code_addr (Thumb bit)
        ];
        let header_dst = LetterPatchLayout::BUF_START as *mut u32;
        core::ptr::copy_nonoverlapping(header.as_ptr(), header_dst, 3);

        // 2. 清零补丁代码区
        let code_dst = LetterPatchLayout::CODE_START as *mut u8;
        core::ptr::write_bytes(code_dst, 0, LetterPatchLayout::CODE_SIZE);

        // 3. 复制补丁代码
        let code_dst = LetterPatchLayout::CODE_START as *mut u32;
        core::ptr::copy_nonoverlapping(bin.as_ptr(), code_dst, bin.len());
    }

    info!("Letter Series patch installed successfully");
    Ok(())
}
