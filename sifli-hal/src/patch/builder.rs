//! Builder 模式的安装器实现。

use super::{
    config::Config,
    entry::PatchEntry,
    error::Error,
    types::ChannelMask,
    Patch,
};
use crate::syscfg::PatchType;

/// 安装结果报告
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallReport {
    /// 实际启用的通道掩码
    pub mask: ChannelMask,
    /// 安装的条目数量
    pub count: usize,
}

impl InstallReport {
    /// 创建新的报告
    pub const fn new(mask: ChannelMask, count: usize) -> Self {
        Self { mask, count }
    }

    /// 是否成功安装了至少一个补丁
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.mask.is_empty()
    }

    /// 启用的通道数量
    #[inline]
    pub const fn channel_count(&self) -> u32 {
        self.mask.count()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for InstallReport {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "InstallReport {{ mask: {:?}, count: {=usize} }}",
            self.mask,
            self.count
        );
    }
}

/// 数据安装结果（包含额外验证信息）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataInstallReport {
    /// 基本安装报告
    pub report: InstallReport,
    /// 补丁代码区第一个字（用于验证数据已写入）
    pub first_code_word: u32,
}

#[cfg(feature = "defmt")]
impl defmt::Format for DataInstallReport {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "DataInstallReport {{ report: {:?}, first_code_word: {=u32:#010x} }}",
            self.report,
            self.first_code_word
        );
    }
}

/// 安装源 trait
pub trait InstallSource {
    /// 安装到指定的 PATCH 外设
    fn install_to(self, patch: &mut Patch, config: Config) -> Result<InstallReport, Error>;
}

/// 补丁安装构建器
pub struct InstallBuilder<'p, 'd, S: InstallSource> {
    pub(super) patch: &'p mut Patch<'d>,
    pub(super) source: S,
    pub(super) config: Config,
}

impl<'p, 'd, S: InstallSource> InstallBuilder<'p, 'd, S> {
    /// 设置配置
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// 禁用验证（不推荐）
    pub fn skip_verification(mut self) -> Self {
        self.config.verify = false;
        self
    }

    /// 执行安装
    pub fn install(self) -> Result<InstallReport, Error> {
        self.source.install_to(self.patch, self.config)
    }
}

/// 特化：从条目安装
impl<'p, 'd> InstallBuilder<'p, 'd, EntriesSource<'p>> {
    /// 指定通道掩码（用于恢复）
    pub fn with_mask(self, mask: ChannelMask) -> InstallBuilder<'p, 'd, EntriesWithMask<'p>> {
        InstallBuilder {
            patch: self.patch,
            source: EntriesWithMask {
                entries: self.source.entries,
                mask,
            },
            config: self.config,
        }
    }
}

/// 从条目数组安装
pub struct EntriesSource<'a> {
    pub(super) entries: &'a [PatchEntry],
}

impl InstallSource for EntriesSource<'_> {
    fn install_to(self, patch: &mut Patch, config: Config) -> Result<InstallReport, Error> {
        patch.apply_entries(self.entries, None, config.verify)
    }
}

/// 从条目+掩码安装（恢复场景）
pub struct EntriesWithMask<'a> {
    pub(super) entries: &'a [PatchEntry],
    pub(super) mask: ChannelMask,
}

impl InstallSource for EntriesWithMask<'_> {
    fn install_to(self, patch: &mut Patch, config: Config) -> Result<InstallReport, Error> {
        patch.apply_entries(self.entries, Some(self.mask), config.verify)
    }
}

/// 从内存数据安装
pub struct DataSource<'a> {
    pub(super) record: &'a [u32],
    pub(super) code: &'a [u32],
}

impl InstallSource for DataSource<'_> {
    fn install_to(self, patch: &mut Patch, config: Config) -> Result<InstallReport, Error> {
        patch.install_from_data_internal(self.record, self.code, config.verify)
    }
}

/// 自动选择补丁版本的源
pub struct AutoSource<'a> {
    pub(super) patch_type: Option<PatchType>,
    pub(super) revid: u8,
    pub(super) a3_record: &'a [u32],
    pub(super) a3_code: &'a [u32],
    pub(super) letter_record: &'a [u32],
    pub(super) letter_code: &'a [u32],
}

impl InstallSource for AutoSource<'_> {
    fn install_to(self, patch: &mut Patch, config: Config) -> Result<InstallReport, Error> {
        let patch_type = self
            .patch_type
            .ok_or(Error::UnsupportedRevision { revid: self.revid })?;

        let (record, code) = match patch_type {
            PatchType::A3 => (self.a3_record, self.a3_code),
            PatchType::LetterSeries => (self.letter_record, self.letter_code),
        };

        patch.install_from_data_internal(record, code, config.verify)
    }
}
