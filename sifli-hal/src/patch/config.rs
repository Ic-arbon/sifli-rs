//! PATCH 操作配置。

/// PATCH 操作配置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Config {
    /// 是否验证 CER 寄存器（默认 true）
    ///
    /// 启用后，安装完成会读回 CER 寄存器确认写入成功。
    /// 禁用可以略微提升性能，但不推荐。
    pub verify: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { verify: true }
    }
}

impl Config {
    /// 创建默认配置
    pub const fn new() -> Self {
        Self { verify: true }
    }

    /// 禁用验证
    pub const fn without_verification(mut self) -> Self {
        self.verify = false;
        self
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Config {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Config {{ verify: {=bool} }}",
            self.verify
        );
    }
}
