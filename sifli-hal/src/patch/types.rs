//! 补丁模块的类型安全包装类型。

use core::fmt;

/// PATCH 支持的通道数量。
pub const CHANNEL_COUNT: usize = 32;
/// 通道地址字段可用的位宽（不含低 2 位对齐位）。
pub const ADDRESS_BITS: u32 = 0x1f_fff;

/// 通道掩码（32-bit，每个 bit 代表一个通道）
///
/// 使用 `#[repr(transparent)]` 确保零成本抽象。
#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ChannelMask(u32);

impl ChannelMask {
    /// 空掩码（所有通道禁用）
    pub const EMPTY: Self = Self(0);

    /// 全掩码（所有 32 通道启用）
    pub const ALL: Self = Self(u32::MAX);

    /// 从原始位创建掩码
    #[inline]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// 获取原始位值
    #[inline]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// 检查是否为空（无通道启用）
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// 启用的通道数量
    #[inline]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// 检查指定通道是否启用
    #[inline]
    pub const fn contains(self, channel: usize) -> bool {
        (self.0 & (1 << channel)) != 0
    }

    /// 迭代所有启用的通道索引
    pub fn enabled_channels(self) -> impl Iterator<Item = usize> {
        (0..32).filter(move |&ch| self.contains(ch))
    }
}

impl fmt::Debug for ChannelMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChannelMask({:#010x}, {} enabled)", self.0, self.count())
    }
}

impl fmt::Display for ChannelMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} channel(s) enabled", self.count())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ChannelMask {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ChannelMask({=u32:#010x}, {=u32} enabled)",
            self.0,
            self.count()
        );
    }
}

/// ROM 地址（必须 4 字节对齐，且在有效范围内）
///
/// 使用 `#[repr(transparent)]` 确保零成本抽象。
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct RomAddress(u32);

impl RomAddress {
    /// 创建 ROM 地址（编译时检查）
    ///
    /// # Errors
    ///
    /// - `AddressError::Unaligned` 如果地址未 4 字节对齐
    /// - `AddressError::OutOfRange` 如果地址超出硬件可寻址范围
    #[inline]
    pub const fn new(addr: u32) -> Result<Self, AddressError> {
        if addr & 0x3 != 0 {
            Err(AddressError::Unaligned)
        } else if (addr >> 2) > ADDRESS_BITS {
            Err(AddressError::OutOfRange)
        } else {
            Ok(Self(addr))
        }
    }

    /// 不检查创建（用于常量，调用者需确保有效性）
    ///
    /// # Safety
    ///
    /// 调用者必须确保地址有效（4 字节对齐且在范围内）。
    #[inline]
    pub const unsafe fn new_unchecked(addr: u32) -> Self {
        Self(addr)
    }

    /// 获取原始地址值
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for RomAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RomAddress({:#010x})", self.0)
    }
}

impl fmt::Display for RomAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#010x}", self.0)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for RomAddress {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "RomAddress({=u32:#010x})", self.0);
    }
}

/// ROM 地址错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    /// 地址未按 4 字节对齐
    Unaligned,
    /// 地址超出硬件可寻址范围
    OutOfRange,
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressError::Unaligned => write!(f, "address not 4-byte aligned"),
            AddressError::OutOfRange => write!(f, "address out of hardware range"),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for AddressError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            AddressError::Unaligned => defmt::write!(fmt, "AddressError::Unaligned"),
            AddressError::OutOfRange => defmt::write!(fmt, "AddressError::OutOfRange"),
        }
    }
}
