# SiFli HAL

[![Crates.io][badge-license]][crates]
[![Crates.io][badge-version]][crates]
[![docs.rs][badge-docsrs]][docsrs]
[![Support status][badge-support-status]][githubrepo]

[badge-license]: https://img.shields.io/crates/l/sifli-hal?style=for-the-badge
[badge-version]: https://img.shields.io/crates/v/sifli-hal?style=for-the-badge
[badge-docsrs]: https://img.shields.io/docsrs/sifli-hal?style=for-the-badge
[badge-support-status]: https://img.shields.io/badge/Support_status-Community-yellow?style=for-the-badge
[crates]: https://crates.io/crates/sifli-hal
[docsrs]: https://docs.rs/sifli-hal
[githubrepo]: https://github.com/OpenSiFli/sifli-hal

[English](README.md) | 中文

SiFli MCU的Rust硬件抽象层(HAL)和[Embassy](https://github.com/embassy-rs/embassy)驱动。

> [!WARNING]
> 
> 此project仍在开发中，尚未准备好用于生产环境。

## 快速开始！

[嵌入式Rust介绍](../docs/intro_to_embedded_rust.md)

[开始使用](../docs/get_started.md)

[烧录与调试指南](../docs/flash_and_debug.md)

## 当前状态

| 系列                 | SF32LB52x        |
| -------------------- | ---------------- |
| Embassy              | ✅+               |
| RCC                  | ✅                |
| GPIO                 | ✅                |
| INTERRUPT            | ✅                |
| PINMUX (类型系统)    | ✅                |
| PMU                  | 仅DVFS切换       |
| DMA                  |                  |
| USART                | ✅+               |
| I2C                  |                  |
| SPI                  |                  |
| 蓝牙                 |                  |
| USB                  |                  |
| ePicasso             |                  |

- ✅ : 已实现
- 空白 : 未实现
- ❓ : 需要示例验证
- `+` : 支持异步
- N/A : 不可用

## Features

- `defmt`, `log`: 调试日志输出。

- `sf32lb52x`: 目标芯片选择。目前仅支持`sf32lb52x`。

- `set-msplim`: 在`__pre_init`中设置MSPLIM寄存器。此寄存器必须在主函数的栈设置前配置（因为引导加载程序可能已将其配置为不同的值），否则将导致硬故障[SiFli-SDK #32](https://github.com/OpenSiFli/SiFli-SDK/issues/32)。

  该特性将在[cortex-m-rt #580](https://github.com/rust-embedded/cortex-m/pull/580)发布后移除。

- `time-driver-xxx`: 为`time-driver`配置定时器。它至少需要两个捕获/比较通道。对于`sf32lb52x hcpu`，只有`atim1`（TODO：[#5](https://github.com/OpenSiFli/sifli-rs/issues/5)）、`gptim1`和`gptim2`可用。

- `unchecked-overclocking`: 启用此特性以禁用超频检查。除非你知道自己在做什么，否则不要启用此特性。

## 许可证

本项目采用以下两种许可证之一：

- Apache许可证2.0版 ([LICENSE-APACHE](../LICENSE-APACHE) 或 <http://www.apache.org/licenses/LICENSE-2.0>)

- MIT许可证 ([LICENSE-MIT](../LICENSE-MIT) 或 <http://opensource.org/licenses/MIT>)

由您选择。