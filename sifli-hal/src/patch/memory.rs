//! 补丁 RAM 内存操作（封装 unsafe 操作）。

use core::{mem, ptr};

use super::error::Error;

/// 补丁 RAM（系统总线视角）起始地址。
pub const PATCH_RAM_SYS_BASE: usize = 0x2040_6000;
/// 补丁 RAM 大小（字节）。
pub const PATCH_RAM_TOTAL_SIZE: usize = 0x2000;
/// 补丁记录区容量（字节）。
pub const PATCH_RECORD_SIZE: usize = 0x100;
/// 补丁记录区基地址（系统总线视角）。
pub const PATCH_RECORD_SYS_ADDR: usize = PATCH_RAM_SYS_BASE + PATCH_RAM_TOTAL_SIZE - PATCH_RECORD_SIZE;

/// 记录块头部标识（"PTCH"）。
pub const TAG: u32 = 0x5054_4348;

pub(super) const PATCH_HEADER_WORDS: usize = 2;
pub(super) const PATCH_RECORD_WORDS: usize = PATCH_RECORD_SIZE / mem::size_of::<u32>();
pub(super) const PATCH_CODE_WORDS: usize = PATCH_RAM_TOTAL_SIZE / mem::size_of::<u32>();

/// 写入补丁代码到 RAM
///
/// # Safety
///
/// 调用者需确保补丁 RAM 区域可访问。
pub(super) unsafe fn write_code(code_words: &[u32]) -> Result<(), Error> {
    let code_bytes = code_words.len() * mem::size_of::<u32>();
    if code_bytes > PATCH_RAM_TOTAL_SIZE {
        return Err(Error::CodeOverflow {
            size: code_bytes,
            capacity: PATCH_RAM_TOTAL_SIZE,
        });
    }

    let dest = PATCH_RAM_SYS_BASE as *mut u32;
    ptr::copy_nonoverlapping(code_words.as_ptr(), dest, code_words.len());

    // 清零剩余区域
    let remaining = PATCH_CODE_WORDS - code_words.len();
    ptr::write_bytes(dest.add(code_words.len()), 0, remaining);

    Ok(())
}

/// 写入补丁记录到 RAM
///
/// # Safety
///
/// 调用者需确保补丁 RAM 区域可访问。
pub(super) unsafe fn write_record(record_words: &[u32]) -> Result<(), Error> {
    let record_bytes = record_words.len() * mem::size_of::<u32>();
    if record_bytes > PATCH_RECORD_SIZE {
        return Err(Error::RecordOverflow {
            size: record_bytes,
            capacity: PATCH_RECORD_SIZE,
        });
    }

    let dest = PATCH_RECORD_SYS_ADDR as *mut u32;
    ptr::copy_nonoverlapping(record_words.as_ptr(), dest, record_words.len());

    // 清零剩余区域
    let remaining = PATCH_RECORD_WORDS - record_words.len();
    ptr::write_bytes(dest.add(record_words.len()), 0, remaining);

    Ok(())
}

/// 读取补丁记录区
///
/// # Safety
///
/// 调用者需确保补丁 RAM 区域可访问。
#[inline]
pub(super) unsafe fn read_record() -> &'static [u32] {
    core::slice::from_raw_parts(PATCH_RECORD_SYS_ADDR as *const u32, PATCH_RECORD_WORDS)
}
