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

[中文](README_zh.md) | English

Rust Hardware Abstraction Layer (HAL) and [Embassy](https://github.com/embassy-rs/embassy) driver for SiFli MCUs.

> [!WARNING]
> 
> This project is working-in-progress and not ready for production use.

## Let's GO!

[Introduction to Embedded Rust](../docs/intro_to_embedded_rust.md)

[Get Started](../docs/get_started.md)

[Flash and Debug Guide](../docs/flash_and_debug.md)

## Status

<details open>
<summary><strong>HAL Implementation Status (Click to expand/collapse)</strong></summary>
<div>
  <ul>
    <li>✅: Supported & Tested</li>
    <li>🌗: Partially Supported & Tested</li>
    <li>❓: Written, needs example/test</li>
    <li>📝: Planned & WIP</li>
    <li>❌: Not supported by Hardware (N/A)</li>
    <li>➕: Async Feature</li>
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

- `defmt`, `log`: Debug log output.

- `sf32lb52x`: Target chip selection. Currently, only `sf32lb52x` is supported.

- `set-msplim`: Set the MSPLIM register in `__pre_init`. This register must be set before the main function’s stack setup (since the bootloader may have already configured it to a different value), otherwise, it will cause a HardFault [SiFli-SDK #32](https://github.com/OpenSiFli/SiFli-SDK/issues/32).

  This feature will be removed after [cortex-m-rt #580](https://github.com/rust-embedded/cortex-m/pull/580)  is released.

- `time-driver-xxx`: Timer configuration for `time-driver`. It requires at least two capture/compare channels. For the `sf32lb52x hcpu`, only `atim1` (TODO: [#5](https://github.com/OpenSiFli/sifli-rs/issues/5)), `gptim1`, and `gptim2` are available.

- `unchecked-overclocking`: Enable this feature to disable the overclocking check. DO NOT ENABLE THIS FEATURE UNLESS YOU KNOW WHAT YOU'RE DOING.

## License

This project is under Apache License, Version 2.0 ([LICENSE](../LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).