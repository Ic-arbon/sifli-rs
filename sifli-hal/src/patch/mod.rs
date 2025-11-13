//! ROM Patch 控制模块。
//!
//! 该模块为 SiFli SF32LB52x 系列芯片的 PATCH 外设提供类型安全的 API。

use core::mem;

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::syscfg::Idr;
use crate::{pac, peripherals, Peripheral};

mod builder;
mod config;
mod entry;
mod error;
mod memory;
mod types;

// 重导出公共 API
pub use builder::{
    AutoSource, DataInstallReport, DataSource, EntriesSource, EntriesWithMask, InstallBuilder,
    InstallReport, InstallSource,
};
pub use config::Config;
pub use entry::PatchEntry;
pub use error::Error;
pub use types::{AddressError, ChannelMask, RomAddress, CHANNEL_COUNT};

pub use memory::{PATCH_RAM_SYS_BASE, PATCH_RAM_TOTAL_SIZE, PATCH_RECORD_SIZE, PATCH_RECORD_SYS_ADDR, TAG};

/// Always-on 通道起始索引（SF32LB52x 为 0）。
pub const PATCH_AON: usize = 0;

/// PATCH 外设驱动
pub struct Patch<'d> {
    regs: pac::patch::Patch,
    _peri: PeripheralRef<'d, peripherals::PATCH>,
}

impl<'d> Patch<'d> {
    /// 创建 PATCH 驱动实例
    ///
    /// # Example
    /// ```no_run
    /// let p = sifli_hal::init(Default::default());
    /// let patch = sifli_hal::patch::Patch::new(p.PATCH);
    /// ```
    pub fn new(patch: impl Peripheral<P = peripherals::PATCH> + 'd) -> Self {
        into_ref!(patch);
        Self {
            regs: pac::PATCH,
            _peri: patch,
        }
    }

    //=== 基础硬件操作 ===

    /// 读取通道使能掩码
    ///
    /// 返回当前启用的通道掩码。
    #[inline]
    pub fn channel_mask(&self) -> ChannelMask {
        ChannelMask::from_bits(self.regs.cer().read().ce())
    }

    /// 禁用所有通道
    ///
    /// 清除所有补丁通道配置。
    pub fn disable_all(&mut self) {
        self.regs.cer().write(|w| w.set_ce(0));
        self.regs.csr().write(|w| w.set_cs(0));
    }

    /// PATCH 模块版本号
    #[inline]
    pub fn version(&self) -> u32 {
        self.regs.ver().read().id()
    }

    //=== Builder API（安装入口）===

    /// 使用补丁条目数组安装
    ///
    /// ⚠️ **注意**：此方法仅配置硬件寄存器，不会写入补丁代码到 RAM。
    /// 适用于恢复之前保存的配置，或在补丁代码已通过 `with_data()` 写入后使用。
    ///
    /// # Example
    /// ```no_run
    /// # use sifli_hal::patch::{Patch, PatchEntry};
    /// # let p = sifli_hal::init(Default::default());
    /// # let mut patch = Patch::new(p.PATCH);
    /// # let entries = [];
    /// // 恢复保存的配置
    /// let report = patch
    ///     .with_entries(&entries)
    ///     .install()?;
    /// # Ok::<(), sifli_hal::patch::Error>(())
    /// ```
    pub fn with_entries<'p>(
        &'p mut self,
        entries: &'p [PatchEntry],
    ) -> InstallBuilder<'p, 'd, EntriesSource<'p>> {
        InstallBuilder {
            patch: self,
            source: EntriesSource { entries },
            config: Config::default(),
        }
    }

    /// 使用原始补丁数据安装
    ///
    /// 从 SDK 格式的补丁数据（record + code）安装。会将数据写入补丁 RAM，
    /// 然后配置硬件寄存器。适用于首次安装或系统重启后。
    ///
    /// # Example
    /// ```no_run
    /// # use sifli_hal::patch::Patch;
    /// # let p = sifli_hal::init(Default::default());
    /// # let mut patch = Patch::new(p.PATCH);
    /// # let record: &[u32] = &[]; let code: &[u32] = &[];
    /// // 从 SDK 数据安装
    /// let report = patch
    ///     .with_data(record, code)
    ///     .install()?;
    /// # Ok::<(), sifli_hal::patch::Error>(())
    /// ```
    pub fn with_data<'p>(
        &'p mut self,
        record: &'p [u32],
        code: &'p [u32],
    ) -> InstallBuilder<'p, 'd, DataSource<'p>> {
        InstallBuilder {
            patch: self,
            source: DataSource { record, code },
            config: Config::default(),
        }
    }

    /// 根据芯片版本自动选择补丁数据并安装
    ///
    /// 读取芯片版本号，自动选择对应的补丁数据（A3 或 Letter Series）。
    /// 是最常用的安装方式，推荐用于生产环境。
    ///
    /// # Example
    /// ```no_run
    /// # use sifli_hal::patch::Patch;
    /// # use sifli_hal::syscfg::SysCfg;
    /// # let p = sifli_hal::init(Default::default());
    /// # let mut patch = Patch::new(p.PATCH);
    /// # let a3_rec: &[u32] = &[]; let a3_code: &[u32] = &[];
    /// # let ls_rec: &[u32] = &[]; let ls_code: &[u32] = &[];
    /// // 读取芯片信息并自动选择版本安装
    /// let idr = SysCfg::read_idr();
    /// let report = patch
    ///     .auto_select(&idr, a3_rec, a3_code, ls_rec, ls_code)
    ///     .install()?;
    /// # Ok::<(), sifli_hal::patch::Error>(())
    /// ```
    pub fn auto_select<'p>(
        &'p mut self,
        idr: &Idr,
        a3_record: &'p [u32],
        a3_code: &'p [u32],
        letter_record: &'p [u32],
        letter_code: &'p [u32],
    ) -> InstallBuilder<'p, 'd, AutoSource<'p>> {
        let patch_type = idr.revision().patch_type();

        InstallBuilder {
            patch: self,
            source: AutoSource {
                patch_type,
                revid: idr.revid,
                a3_record,
                a3_code,
                letter_record,
                letter_code,
            },
            config: Config::default(),
        }
    }

    //=== 保存与恢复（零分配）===

    /// 保存当前补丁配置到缓冲区
    ///
    /// 从硬件读取当前激活的补丁配置，保存到用户提供的缓冲区。
    /// 返回 `(条目数量, 通道掩码)`。
    ///
    /// # Example
    /// ```no_run
    /// # use sifli_hal::patch::{Patch, PatchEntry};
    /// # let p = sifli_hal::init(Default::default());
    /// # let patch = Patch::new(p.PATCH);
    /// let mut buffer = [PatchEntry::default(); 32];
    /// let (count, mask) = patch.save(&mut buffer)?;
    /// # Ok::<(), sifli_hal::patch::Error>(())
    /// ```
    pub fn save(&self, buffer: &mut [PatchEntry]) -> Result<(usize, ChannelMask), Error> {
        if buffer.is_empty() {
            return Err(Error::BufferTooSmall {
                required: 1,
                provided: 0,
            });
        }

        let mask = self.channel_mask();
        let mut count = 0;

        for channel in mask.enabled_channels() {
            if count >= buffer.len() {
                return Err(Error::BufferTooSmall {
                    required: mask.count() as usize,
                    provided: buffer.len(),
                });
            }

            // 读取通道配置
            let ch_reg = self.regs.ch(channel).read();
            let break_addr = ch_reg.addr() << 2;

            // 选通通道，读取数据
            self.regs.csr().write(|w| w.set_cs(1 << channel));
            let data = self.regs.cdr().read().data();

            buffer[count] = PatchEntry::new(break_addr, data);
            count += 1;
        }

        // 清除选择寄存器
        self.regs.csr().write(|w| w.set_cs(0));

        Ok((count, mask))
    }

    //=== 内部实现 ===

    /// 应用补丁条目到硬件
    pub(crate) fn apply_entries(
        &mut self,
        entries: &[PatchEntry],
        mask: Option<ChannelMask>,
        verify: bool,
    ) -> Result<InstallReport, Error> {
        // 验证条目数量
        if entries.len() > CHANNEL_COUNT {
            return Err(Error::TooManyEntries {
                count: entries.len(),
                max: CHANNEL_COUNT,
            });
        }

        // 验证所有条目
        for (idx, entry) in entries.iter().enumerate() {
            if let Err(reason) = entry.validate() {
                return Err(Error::InvalidAddress {
                    index: idx,
                    addr: entry.break_addr,
                    reason,
                });
            }
        }

        // 禁用所有通道
        self.disable_all();

        // 写入条目
        let mut applied = ChannelMask::EMPTY;
        for (idx, entry) in entries.iter().enumerate() {
            let channel = idx + PATCH_AON;

            // 如果指定了掩码，检查是否需要启用此通道
            if let Some(m) = mask {
                if !m.contains(channel) {
                    continue;
                }
            }

            self.write_entry(channel, entry);
            applied = ChannelMask::from_bits(applied.bits() | (1 << channel));
        }

        // 启用通道
        let enable_mask = mask.unwrap_or(applied);
        self.regs.cer().write(|w| w.set_ce(enable_mask.bits()));

        // 验证 CER 寄存器
        if verify {
            let actual = self.channel_mask();
            if actual != enable_mask {
                return Err(Error::VerificationFailed {
                    expected: enable_mask,
                    actual,
                });
            }
        }

        Ok(InstallReport::new(applied, entries.len()))
    }

    /// 从数据安装（内部实现）
    pub(crate) fn install_from_data_internal(
        &mut self,
        record_words: &[u32],
        code_words: &[u32],
        verify: bool,
    ) -> Result<InstallReport, Error> {
        // 写入数据到 RAM
        unsafe {
            memory::write_code(code_words)?;
            memory::write_record(record_words)?;
        }

        // 从 RAM 解析并安装
        let record = unsafe { memory::read_record() };
        let entries = parse_record(record)?;

        self.apply_entries(entries, None, verify)
    }

    #[inline]
    fn write_entry(&self, channel: usize, entry: &PatchEntry) {
        let offset = entry.break_addr >> 2;
        self.regs.ch(channel).write(|w| w.set_addr(offset));
        self.regs.csr().write(|w| w.set_cs(1 << channel));
        self.regs.cdr().write(|w| w.set_data(entry.data));
    }
}

/// 解析补丁记录
fn parse_record(words: &[u32]) -> Result<&[PatchEntry], Error> {
    if words.len() < memory::PATCH_HEADER_WORDS {
        return Err(Error::InsufficientData {
            required: memory::PATCH_HEADER_WORDS * mem::size_of::<u32>(),
            available: core::mem::size_of_val(words),
        });
    }

    // 检查标签
    if words[0] != TAG {
        return Err(Error::InvalidRecordTag {
            expected: TAG,
            found: words[0],
        });
    }

    // 解析大小
    let size_bytes = words[1] as usize;
    if !size_bytes.is_multiple_of(mem::size_of::<PatchEntry>()) {
        return Err(Error::MisalignedRecordSize { size: size_bytes });
    }

    let entry_count = size_bytes / mem::size_of::<PatchEntry>();
    let required_words = memory::PATCH_HEADER_WORDS + entry_count * 2;

    if words.len() < required_words {
        return Err(Error::InsufficientData {
            required: required_words * mem::size_of::<u32>(),
            available: core::mem::size_of_val(words),
        });
    }

    // 转换为 PatchEntry 切片
    let entries_ptr = unsafe {
        words
            .as_ptr()
            .add(memory::PATCH_HEADER_WORDS) as *const PatchEntry
    };
    let entries = unsafe { core::slice::from_raw_parts(entries_ptr, entry_count) };

    Ok(entries)
}
