//! ROM Patch 控制模块。
//!
//! 该模块为 SiFli SF32LB52x 系列芯片的 PATCH 外设提供安全包装，
//! 支持在运行时对 ROM 指令进行替换或钩挂。接口设计参考了官方
//! `HAL_PATCH_install2`/`HAL_PATCH_save` 实现。

use core::{mem, ptr, slice};

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::{pac, peripherals, Peripheral};

/// PATCH 支持的通道数量。
pub const CHANNEL_COUNT: usize = 32;
/// 通道地址字段可用的位宽（不含低 2 位对齐位）。
pub const ADDRESS_BITS: u32 = 0x1f_fff;
/// 记录块头部标识（"PTCH"）。
pub const TAG: u32 = 0x5054_4348;
/// Always-on 通道起始索引（SF32LB52x 为 0）。
pub const PATCH_AON: usize = 0;

/// 补丁 RAM（系统总线视角）起始地址。
pub const PATCH_RAM_SYS_BASE: usize = 0x2040_6000;
/// 补丁 RAM 大小（字节）。
pub const PATCH_RAM_TOTAL_SIZE: usize = 0x2000;
/// 补丁记录区容量（字节）。
pub const PATCH_RECORD_SIZE: usize = 0x100;
/// 补丁记录区基地址（系统总线视角）。
pub const PATCH_RECORD_SYS_ADDR: usize =
    PATCH_RAM_SYS_BASE + PATCH_RAM_TOTAL_SIZE - PATCH_RECORD_SIZE;

const PATCH_HEADER_WORDS: usize = 2;
const WORDS_PER_ENTRY: usize = mem::size_of::<PatchEntry>() / mem::size_of::<u32>();
const PATCH_RECORD_WORDS: usize = PATCH_RECORD_SIZE / mem::size_of::<u32>();
const PATCH_CODE_WORDS: usize = PATCH_RAM_TOTAL_SIZE / mem::size_of::<u32>();

const CHANNEL_BITMASK: u32 = if CHANNEL_COUNT >= u32::BITS as usize {
    u32::MAX
} else {
    (1u32 << CHANNEL_COUNT) - 1
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// 补丁描述条目。
pub struct PatchEntry {
    /// 触发补丁的指令地址（必须 4 字节对齐，并位于 ROM 范围内）。
    pub break_addr: u32,
    /// 与 `break_addr` 地址处指令匹配的原始指令值。
    ///
    /// 硬件会在使能补丁前比对该值，确保 ROM 内容未被第三方修改。
    pub data: u32,
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

/// PATCH 操作过程中可能出现的错误。
#[derive(Debug, PartialEq, Eq)]
pub enum PatchError {
    /// 补丁条目数量超过硬件可用通道数。
    TooManyEntries,
    /// `break_addr` 未按 4 字节对齐。
    UnalignedAddr { index: usize, addr: u32 },
    /// `break_addr` 超出硬件可寻址范围。
    AddrOutOfRange { index: usize, addr: u32 },
    /// 提供的缓冲区容量不足以保存当前补丁表。
    BufferTooSmall,
    /// 指定的通道使能掩码包含非法位。
    InvalidMask,
}

#[cfg(feature = "defmt")]
impl defmt::Format for PatchError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            PatchError::TooManyEntries => defmt::write!(fmt, "PatchError::TooManyEntries"),
            PatchError::UnalignedAddr { index, addr } => {
                defmt::write!(
                    fmt,
                    "PatchError::UnalignedAddr {{ index: {=usize}, addr: {=u32:#010x} }}",
                    index,
                    addr
                )
            }
            PatchError::AddrOutOfRange { index, addr } => {
                defmt::write!(
                    fmt,
                    "PatchError::AddrOutOfRange {{ index: {=usize}, addr: {=u32:#010x} }}",
                    index,
                    addr
                )
            }
            PatchError::BufferTooSmall => defmt::write!(fmt, "PatchError::BufferTooSmall"),
            PatchError::InvalidMask => defmt::write!(fmt, "PatchError::InvalidMask"),
        }
    }
}

/// `HAL_PATCH_install` 语义的错误类型。
#[derive(Debug, PartialEq, Eq)]
pub enum InstallError {
    /// 记录头部标识不匹配。
    TagMismatch,
    /// 记录长度超出缓冲区容量。
    SizeOverflow,
    /// 记录数据长度与 `PatchEntry` 大小不对齐。
    MisalignedSize,
    /// 记录数据不足以覆盖所有条目。
    InsufficientData,
    /// 入参指针为空。
    NullPointer,
    /// 写入硬件补丁寄存器时出现错误。
    Patch(PatchError),
}

impl From<PatchError> for InstallError {
    fn from(err: PatchError) -> Self {
        InstallError::Patch(err)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for InstallError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            InstallError::TagMismatch => defmt::write!(fmt, "InstallError::TagMismatch"),
            InstallError::SizeOverflow => defmt::write!(fmt, "InstallError::SizeOverflow"),
            InstallError::MisalignedSize => defmt::write!(fmt, "InstallError::MisalignedSize"),
            InstallError::InsufficientData => {
                defmt::write!(fmt, "InstallError::InsufficientData")
            }
            InstallError::NullPointer => defmt::write!(fmt, "InstallError::NullPointer"),
            InstallError::Patch(err) => defmt::write!(fmt, "InstallError::Patch({:?})", err),
        }
    }
}

/// 补丁安装结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallReport {
    /// 实际写入的补丁使能掩码。
    pub applied_mask: u32,
    /// 记录中解析出的条目数量。
    pub entry_count: usize,
}

/// PATCH 外设安全包装。
pub struct Patch<'d> {
    regs: pac::patch::Patch,
    _peri: PeripheralRef<'d, peripherals::PATCH>,
}

impl<'d> Patch<'d> {
    /// 取得 PATCH 外设单例。
    pub fn new(patch: impl Peripheral<P = peripherals::PATCH> + 'd) -> Self {
        into_ref!(patch);
        Self {
            regs: pac::PATCH,
            _peri: patch,
        }
    }

    /// 从补丁记录区（`mem_map.h` 中的 `LCPU_PATCH_RECORD_ADDR`）解析并安装补丁。
    ///
    /// # Safety
    ///
    /// 调用方需确保 LPSYS 补丁记录区可由当前内核访问，且其中内容遵循 SDK
    /// `PATCH_RECORD_U32` 布局。
    pub unsafe fn install_from_patch_ram(&mut self) -> Result<InstallReport, InstallError> {
        self.install_from_addr(PATCH_RECORD_SYS_ADDR as *const u32)
    }

    /// 将补丁表和补丁代码写入补丁 RAM 并执行安装。
    pub fn install_from_data(
        &mut self,
        record_words: &[u32],
        code_words: &[u32],
    ) -> Result<InstallDataOutcome, InstallDataError> {
        write_code(code_words)?;
        write_record(record_words)?;

        let report =
            unsafe { self.install_from_patch_ram() }.map_err(InstallDataError::Install)?;
        let first_word = unsafe { ptr::read_volatile(PATCH_RAM_SYS_BASE as *const u32) };

        Ok(InstallDataOutcome { report, first_word })
    }

    /// 按 HAL 实现从指定内存地址解析补丁记录并安装。
    ///
    /// `record_addr` 需指向以 `TAG` 开头、长度字段随后（单位：字节）的补丁表。
    ///
    /// # Safety
    ///
    /// - 指针必须可读，且至少包含 `PATCH_RECORD_SIZE` 字节有效数据；
    /// - 补丁记录的结构须与 SDK 保持一致。
    pub unsafe fn install_from_addr(
        &mut self,
        record_addr: *const u32,
    ) -> Result<InstallReport, InstallError> {
        if record_addr.is_null() {
            return Err(InstallError::NullPointer);
        }

        let record_words = slice::from_raw_parts(record_addr, PATCH_RECORD_WORDS);
        let parse_result = parse_patch_record(record_words)?;
        let entries_ptr = record_words
            .as_ptr()
            .add(PATCH_HEADER_WORDS) as *const PatchEntry;
        let entries = slice::from_raw_parts(entries_ptr, parse_result.entry_count);

        let applied_mask = self.apply(entries).map_err(InstallError::from)?;

        Ok(InstallReport {
            applied_mask,
            entry_count: parse_result.entry_count,
        })
    }

    /// 安装补丁，返回实际写入的通道掩码。
    pub fn apply(&mut self, entries: &[PatchEntry]) -> Result<u32, PatchError> {
        self.apply_internal(entries, 0)
    }

    /// 按照给定掩码恢复补丁，通常用于休眠唤醒后的还原场景。
    pub fn apply_with_mask(
        &mut self,
        entries: &[PatchEntry],
        cer: u32,
    ) -> Result<u32, PatchError> {
        if cer & !CHANNEL_BITMASK != 0 {
            return Err(PatchError::InvalidMask);
        }
        self.apply_internal(entries, cer)
    }

    /// 保存当前补丁表到用户缓冲区，返回 `(实际记录条目, CER 值)`。
    pub fn save(&self, buffer: &mut [PatchEntry]) -> Result<(usize, u32), PatchError> {
        if buffer.is_empty() {
            return Err(PatchError::BufferTooSmall);
        }

        let cer = self.cer();
        let mut count = 0usize;

        let limit = PATCH_AON.saturating_add(buffer.len());
        for channel in PATCH_AON..CHANNEL_COUNT.min(limit) {
            if (cer & (1u32 << channel)) == 0 {
                break;
            }

            let ch = self.regs.ch(channel).read();
            let break_addr = ch.addr() << 2;

            if count >= buffer.len() {
                return Err(PatchError::BufferTooSmall);
            }

            buffer[count] = PatchEntry {
                break_addr,
                // HCPU 无法直接读取 LCPU ROM 指令，保留占位值。
                data: 0,
            };
            count += 1;
        }

        Ok((count, cer))
    }

    /// 关闭所有补丁通道。
    pub fn disable_all(&mut self) {
        self.regs.cer().write(|w| w.set_bits(0));
        self.regs.csr().write(|w| w.set_bits(0));
    }

    /// 返回当前使能掩码。
    #[inline]
    pub fn cer(&self) -> u32 {
        self.regs.cer().read().bits()
    }

    /// 返回当前状态寄存器值。
    #[inline]
    pub fn csr(&self) -> u32 {
        self.regs.csr().read().bits()
    }

    /// 模块版本号。
    #[inline]
    pub fn version(&self) -> u32 {
        self.regs.ver().read().bits()
    }

    fn apply_internal(
        &mut self,
        entries: &[PatchEntry],
        cer_mask: u32,
    ) -> Result<u32, PatchError> {
        let offset = if cer_mask != 0 { PATCH_AON } else { 0 };

        if offset + entries.len() > CHANNEL_COUNT {
            return Err(PatchError::TooManyEntries);
        }

        for (idx, entry) in entries.iter().enumerate() {
            validate_entry(idx + offset, entry)?;
        }

        self.disable_all();

        let mut applied_mask = 0u32;

        for (idx, entry) in entries.iter().enumerate() {
            let channel = idx + offset;
            let bit = 1u32 << channel;

            if cer_mask != 0 && (cer_mask & bit) == 0 {
                continue;
            }

            self.write_entry(channel, entry);
            applied_mask |= bit;
        }

        // 配置完成后清空 CSR，避免残留通道选择。
        self.regs.csr().write(|w| w.set_bits(0));

        let enable_mask = if cer_mask != 0 { cer_mask } else { applied_mask };
        self.regs.cer().write(|w| w.set_bits(enable_mask));

        Ok(applied_mask)
    }

    fn write_entry(&self, channel: usize, entry: &PatchEntry) {
        let offset = entry.break_addr >> 2;

        self.regs.ch(channel).write(|w| {
            w.set_addr(offset);
        });

        let bit = 1u32 << channel;
        self.regs.csr().write(|w| w.set_bits(bit));
        self.regs.cdr().write(|w| w.set_bits(entry.data));
    }
}

struct ParsedRecord {
    entry_count: usize,
}

/// `install_from_data` 附加返回信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallDataOutcome {
    pub report: InstallReport,
    pub first_word: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstallDataError {
    RecordTooLarge { bytes: usize },
    CodeTooLarge { bytes: usize },
    Install(InstallError),
}

impl From<InstallError> for InstallDataError {
    fn from(err: InstallError) -> Self {
        InstallDataError::Install(err)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for InstallDataError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            InstallDataError::RecordTooLarge { bytes } => {
                defmt::write!(fmt, "InstallDataError::RecordTooLarge {{ bytes: {=usize} }}", bytes)
            }
            InstallDataError::CodeTooLarge { bytes } => {
                defmt::write!(fmt, "InstallDataError::CodeTooLarge {{ bytes: {=usize} }}", bytes)
            }
            InstallDataError::Install(err) => {
                defmt::write!(fmt, "InstallDataError::Install({:?})", err)
            }
        }
    }
}

fn parse_patch_record(words: &[u32]) -> Result<ParsedRecord, InstallError> {
    if words.len() < PATCH_HEADER_WORDS {
        return Err(InstallError::InsufficientData);
    }

    if words[0] != TAG {
        return Err(InstallError::TagMismatch);
    }

    let size_bytes = words[1] as usize;
    let available_bytes =
        words.len().saturating_sub(PATCH_HEADER_WORDS) * mem::size_of::<u32>();

    if size_bytes > available_bytes {
        return Err(InstallError::SizeOverflow);
    }

    if size_bytes % mem::size_of::<PatchEntry>() != 0 {
        return Err(InstallError::MisalignedSize);
    }

    let entry_count = size_bytes / mem::size_of::<PatchEntry>();
    let required_words = PATCH_HEADER_WORDS + entry_count * WORDS_PER_ENTRY;
    if words.len() < required_words {
        return Err(InstallError::InsufficientData);
    }

    Ok(ParsedRecord { entry_count })
}

fn write_record(record_words: &[u32]) -> Result<(), InstallDataError> {
    let words = record_words.len();
    if words > PATCH_RECORD_WORDS {
        return Err(InstallDataError::RecordTooLarge {
            bytes: words * mem::size_of::<u32>(),
        });
    }

    let dst = PATCH_RECORD_SYS_ADDR as *mut u32;
    unsafe {
        for (idx, value) in record_words.iter().enumerate() {
            ptr::write_volatile(dst.add(idx), *value);
        }
        for idx in words..PATCH_RECORD_WORDS {
            ptr::write_volatile(dst.add(idx), 0);
        }
    }

    Ok(())
}

fn write_code(code_words: &[u32]) -> Result<(), InstallDataError> {
    let words = code_words.len();
    if words > PATCH_CODE_WORDS {
        return Err(InstallDataError::CodeTooLarge {
            bytes: words * mem::size_of::<u32>(),
        });
    }

    let dest = PATCH_RAM_SYS_BASE as *mut u32;
    unsafe {
        for (idx, value) in code_words.iter().enumerate() {
            ptr::write_volatile(dest.add(idx), *value);
        }
        for idx in words..PATCH_CODE_WORDS {
            ptr::write_volatile(dest.add(idx), 0);
        }
    }

    Ok(())
}

fn validate_entry(index: usize, entry: &PatchEntry) -> Result<(), PatchError> {
    if index >= CHANNEL_COUNT {
        return Err(PatchError::TooManyEntries);
    }
    if entry.break_addr & 0x3 != 0 {
        return Err(PatchError::UnalignedAddr {
            index,
            addr: entry.break_addr,
        });
    }
    if !channel_addr_valid(entry.break_addr) {
        return Err(PatchError::AddrOutOfRange {
            index,
            addr: entry.break_addr,
        });
    }
    Ok(())
}

fn channel_addr_valid(addr: u32) -> bool {
    (addr >> 2) <= ADDRESS_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_address_validation() {
        assert!(channel_addr_valid(0));
        assert!(channel_addr_valid(ADDRESS_BITS << 2));
        assert!(!channel_addr_valid(((ADDRESS_BITS + 1) << 2)));
    }

    #[test]
    fn validate_entry_alignment() {
        let entry = PatchEntry {
            break_addr: 0x1000,
            data: 0,
        };
        assert!(validate_entry(0, &entry).is_ok());

        let bad = PatchEntry {
            break_addr: 0x1001,
            data: 0,
        };
        assert!(matches!(
            validate_entry(0, &bad),
            Err(PatchError::UnalignedAddr { .. })
        ));
    }
}
