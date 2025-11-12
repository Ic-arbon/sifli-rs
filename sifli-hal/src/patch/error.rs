//! PATCH 模块统一错误类型。

use core::fmt;

use super::types::{AddressError, ChannelMask};

/// PATCH 模块统一错误类型
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// 补丁条目数量超过硬件通道数
    TooManyEntries {
        /// 请求的条目数量
        count: usize,
        /// 最大支持的通道数
        max: usize,
    },

    /// 补丁条目地址无效
    InvalidAddress {
        /// 出错的条目索引
        index: usize,
        /// 无效的地址值
        addr: u32,
        /// 错误原因
        reason: AddressError,
    },

    /// 保存缓冲区太小
    BufferTooSmall {
        /// 所需的缓冲区大小
        required: usize,
        /// 实际提供的大小
        provided: usize,
    },

    /// CER 寄存器验证失败
    VerificationFailed {
        /// 期望的通道掩码
        expected: ChannelMask,
        /// 实际读取的掩码
        actual: ChannelMask,
    },

    /// 补丁记录标签不匹配
    InvalidRecordTag {
        /// 期望的标签值
        expected: u32,
        /// 实际读取的标签值
        found: u32,
    },

    /// 记录大小不对齐
    MisalignedRecordSize {
        /// 不对齐的大小
        size: usize,
    },

    /// 记录数据溢出
    RecordOverflow {
        /// 请求的大小
        size: usize,
        /// RAM 容量
        capacity: usize,
    },

    /// 补丁代码溢出
    CodeOverflow {
        /// 请求的大小
        size: usize,
        /// RAM 容量
        capacity: usize,
    },

    /// 记录数据不足
    InsufficientData {
        /// 需要的字节数
        required: usize,
        /// 可用的字节数
        available: usize,
    },

    /// 不支持的芯片版本
    UnsupportedRevision {
        /// 芯片版本 ID
        revid: u8,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TooManyEntries { count, max } => {
                write!(f, "too many patch entries: {} > max {}", count, max)
            }
            Error::InvalidAddress {
                index,
                addr,
                reason,
            } => {
                write!(
                    f,
                    "invalid address at entry[{}]: {:#010x} ({})",
                    index, addr, reason
                )
            }
            Error::BufferTooSmall { required, provided } => {
                write!(
                    f,
                    "buffer too small: required {}, provided {}",
                    required, provided
                )
            }
            Error::VerificationFailed { expected, actual } => {
                write!(
                    f,
                    "CER verification failed: expected {}, got {}",
                    expected, actual
                )
            }
            Error::InvalidRecordTag { expected, found } => {
                write!(
                    f,
                    "invalid record tag: expected {:#010x}, found {:#010x}",
                    expected, found
                )
            }
            Error::MisalignedRecordSize { size } => {
                write!(f, "record size not aligned: {} bytes", size)
            }
            Error::RecordOverflow { size, capacity } => {
                write!(
                    f,
                    "record RAM overflow: {} bytes > capacity {}",
                    size, capacity
                )
            }
            Error::CodeOverflow { size, capacity } => {
                write!(
                    f,
                    "code RAM overflow: {} bytes > capacity {}",
                    size, capacity
                )
            }
            Error::InsufficientData {
                required,
                available,
            } => {
                write!(
                    f,
                    "insufficient data: required {} bytes, available {}",
                    required, available
                )
            }
            Error::UnsupportedRevision { revid } => {
                write!(f, "unsupported chip revision: {:#04x}", revid)
            }
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Error {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Error::TooManyEntries { count, max } => {
                defmt::write!(
                    fmt,
                    "Error::TooManyEntries {{ count: {=usize}, max: {=usize} }}",
                    count,
                    max
                );
            }
            Error::InvalidAddress {
                index,
                addr,
                reason,
            } => {
                defmt::write!(
                    fmt,
                    "Error::InvalidAddress {{ index: {=usize}, addr: {=u32:#010x}, reason: {:?} }}",
                    index,
                    addr,
                    reason
                );
            }
            Error::BufferTooSmall { required, provided } => {
                defmt::write!(
                    fmt,
                    "Error::BufferTooSmall {{ required: {=usize}, provided: {=usize} }}",
                    required,
                    provided
                );
            }
            Error::VerificationFailed { expected, actual } => {
                defmt::write!(
                    fmt,
                    "Error::VerificationFailed {{ expected: {:?}, actual: {:?} }}",
                    expected,
                    actual
                );
            }
            Error::InvalidRecordTag { expected, found } => {
                defmt::write!(
                    fmt,
                    "Error::InvalidRecordTag {{ expected: {=u32:#010x}, found: {=u32:#010x} }}",
                    expected,
                    found
                );
            }
            Error::MisalignedRecordSize { size } => {
                defmt::write!(
                    fmt,
                    "Error::MisalignedRecordSize {{ size: {=usize} }}",
                    size
                );
            }
            Error::RecordOverflow { size, capacity } => {
                defmt::write!(
                    fmt,
                    "Error::RecordOverflow {{ size: {=usize}, capacity: {=usize} }}",
                    size,
                    capacity
                );
            }
            Error::CodeOverflow { size, capacity } => {
                defmt::write!(
                    fmt,
                    "Error::CodeOverflow {{ size: {=usize}, capacity: {=usize} }}",
                    size,
                    capacity
                );
            }
            Error::InsufficientData {
                required,
                available,
            } => {
                defmt::write!(
                    fmt,
                    "Error::InsufficientData {{ required: {=usize}, available: {=usize} }}",
                    required,
                    available
                );
            }
            Error::UnsupportedRevision { revid } => {
                defmt::write!(
                    fmt,
                    "Error::UnsupportedRevision {{ revid: {=u8:#04x} }}",
                    revid
                );
            }
        }
    }
}
