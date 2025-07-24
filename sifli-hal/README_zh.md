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

[入门指南](../docs/get_started.md)

[例程](examples)

[烧录与调试指南](../docs/flash_and_debug.md)

## 当前状态

<details open>
<summary><strong>HAL 实现状态 (点击展开/折叠)</strong></summary>
<div>
  <ul>
    <li>✅: 支持 & 已测试</li>
    <li>🌗: 部分支持 & 已测试</li>
    <li>❓: 已编写, 需要示例/测试</li>
    <li>📝: 计划中 & 开发中</li>
    <li>❌: 硬件不支持 (N/A)</li>
    <li>➕: 异步功能</li>
  </ul>
</div>
<table style="border-collapse: collapse; width: 100%;">
  <thead>
    <tr style="text-align: center;">
      <th style="border: 1px solid #ddd; padding: 8px;" rowspan="2">Peripheral</th>
      <th style="border: 1px solid #ddd; padding: 8px;" rowspan="2">Feature</th>
      <th style="border: 1px solid #ddd; padding: 8px;" colspan="2">sf32lb52x</th>
      <th style="border: 1px solid #ddd; padding: 8px;" rowspan="2">56x</th>
      <th style="border: 1px solid #ddd; padding: 8px;" rowspan="2">58x</th>
    </tr>
    <tr style="text-align: center;">
      <th style="border: 1px solid #ddd; padding: 8px;">hcpu</th>
      <th style="border: 1px solid #ddd; padding: 8px;">lcpu</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>PAC (Peripheral Access Crate)</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">🌗</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>Startup</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>Flash Table</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">🌗</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>Interrupt</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" rowspan="4"><strong>embassy</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px;">GPTIM Time Driver</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">ATIM Time Driver</td>
        <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">❓<a href="https://github.com/OpenSiFli/sifli-rs/issues/5">(#5)</a></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Embassy-style Log (fmt.rs)</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Embassy Peripheral Singleton</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" rowspan="3"><strong>RCC</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px;">Peripheral RCC Codegen (enable, freq...)</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Read current RCC tree</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Modify frequency in same DVFS mode</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" rowspan="4"><strong>PMU</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px;">DVFS Upscale</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">DVFS Downscale</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Charge Modoule</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Buck & LDO</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" rowspan="4"><strong>GPIO</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px;">Blinky</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">PinMux Codegen & AF Config</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">IO Mode & AonPE Config</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">🌗</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">EXTI ➕</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" rowspan="3"><strong>USART</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px;">Blocking</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Buffered(Interrupt) ➕</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">DMA ➕</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" rowspan="8"><strong>GPADC</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px;">Blocking</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Interrupt ➕</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Timer Trigger</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">VBAT & External Channel</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">✅</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Multi Channel & Slot</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Differential Input</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">DMA ➕</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;">Calibration</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>DMA</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>I2C</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>SPI</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>I2S</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>Mailbox</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>BT</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>BLE</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>USB</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">📝</td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
    <tr>
      <td style="border: 1px solid #ddd; padding: 8px;" colspan="2"><strong>ePicasso</strong></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
      <td style="border: 1px solid #ddd; padding: 8px; text-align: center;"></td>
    </tr>
  </tbody>
</table>
</details>

## Features

- `defmt`, `log`: 调试日志输出。

- `sf32lb52x`: 目标芯片选择。目前仅支持`sf32lb52x`。

- `set-msplim`: 在`__pre_init`中设置MSPLIM寄存器。此寄存器必须在主函数的栈设置前配置（因为引导加载程序可能已将其配置为不同的值），否则将导致Hard Fault [SiFli-SDK #32](https://github.com/OpenSiFli/SiFli-SDK/issues/32)。

  该feature将在[cortex-m-rt #580](https://github.com/rust-embedded/cortex-m/pull/580)发布后移除。

- `time-driver-xxx`: 为`time-driver`配置定时器。它至少需要两个捕获/比较通道。对于`sf32lb52x hcpu`，只有`atim1`（TODO：[#5](https://github.com/OpenSiFli/sifli-rs/issues/5)）、`gptim1`和`gptim2`可用。

- `unchecked-overclocking`: 启用此feature以禁用超频检查。除非你知道自己在做什么，否则不要启用此feature!

## 许可证

本项目采用 Apache 2.0许可证（[LICENSE](../LICENSE) 或 <http://www.apache.org/licenses/LICENSE-2.0>）。