//! 补丁条目类型定义。

use super::types::{AddressError, RomAddress};

/// 补丁描述条目
///
/// 每个条目描述一个需要打补丁的 ROM 指令位置。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PatchEntry {
    /// 触发补丁的指令地址（必须 4 字节对齐，并位于 ROM 范围内）
    pub break_addr: u32,
    /// 与 `break_addr` 地址处指令匹配的原始指令值
    ///
    /// 硬件会在使能补丁前比对该值，确保 ROM 内容未被第三方修改。
    pub data: u32,
}

impl PatchEntry {
    /// 创建新的补丁条目
    #[inline]
    pub const fn new(break_addr: u32, data: u32) -> Self {
        Self { break_addr, data }
    }

    /// 验证条目的地址是否有效
    ///
    /// 检查地址对齐和范围。
    pub fn validate(&self) -> Result<(), AddressError> {
        RomAddress::new(self.break_addr)?;
        Ok(())
    }

    /// 获取类型安全的 ROM 地址（假设已验证）
    ///
    /// # Safety
    ///
    /// 调用者需确保此条目已通过 `validate()` 验证。
    #[inline]
    pub const unsafe fn rom_address_unchecked(&self) -> RomAddress {
        RomAddress::new_unchecked(self.break_addr)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PatchEntry {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "PatchEntry {{ break_addr: {=u32:#010x}, data: {=u32:#010x} }}",
            self.break_addr,
            self.data
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_validation() {
        // 有效地址
        let entry = PatchEntry::new(0x1000, 0xAABBCCDD);
        assert!(entry.validate().is_ok());

        // 未对齐地址
        let bad_entry = PatchEntry::new(0x1001, 0);
        assert_eq!(bad_entry.validate(), Err(AddressError::Unaligned));

        // 超出范围地址
        let out_of_range = PatchEntry::new(0xFFFF_FFFF, 0);
        assert_eq!(out_of_range.validate(), Err(AddressError::OutOfRange));
    }
}
