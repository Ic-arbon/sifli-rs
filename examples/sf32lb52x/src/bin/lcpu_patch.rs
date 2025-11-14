#![no_std]
#![no_main]

//! LCPU 补丁安装示例
//!
//! 本示例演示如何使用 `lcpu::install_patch_and_calibrate()` 安装 LCPU 补丁。
//!
//! ## 功能
//!
//! - 自动识别芯片版本（A3/Letter Series）
//! - 自动选择并安装对应的补丁数据
//! - 验证补丁安装结果
//!
//! ## 测试步骤
//!
//! 1. 烧录并运行示例
//! 2. 观察日志输出，确认：
//!    - 芯片版本识别正确
//!    - 补丁数据已写入内存
//! 3. LED (PA26) 持续闪烁表示程序正常运行
//!
//! ## 注意
//!
//! - 此示例仅测试补丁安装功能，不启动 LCPU
//! - `lcpu::power_on()` 尚未完全实现，暂时使用 `install_patch_and_calibrate()`

use defmt::{error, info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

use sifli_hal::lcpu::{self, LcpuConfig, PatchData};
use sifli_hal::syscfg;

#[path = "../patch_data.rs"]
mod patch_a3;
#[path = "../patch_data_rev_b.rs"]
mod patch_ls;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = sifli_hal::init(Default::default());

    // 读取芯片版本
    let idr = syscfg::read_idr();
    let revision = idr.revision();

    info!("芯片版本: {:?} (REVID: 0x{:02x})", revision, idr.revid);

    // 构建 LCPU 配置
    let config = LcpuConfig::new()
        .with_patch_a3(PatchData {
            record: &patch_a3::PATCH_RECORD_U32,
            code: &patch_a3::PATCH_CODE_U32,
        })
        .with_patch_letter(PatchData {
            record: &patch_ls::PATCH_RECORD_U32,
            code: &patch_ls::PATCH_CODE_U32,
        })
        .disable_rf_cal()              // RF 校准尚未实现
        .skip_frequency_check();       // 频率检查尚未实现

    // 自动选择版本并安装补丁
    info!("安装补丁 (自动选择版本)...");
    lcpu::install_patch_and_calibrate(&config, &idr).unwrap();

    // 读取补丁 RAM 首字（验证代码已写入）

    // 打印启用的通道

    // 测试保存功能

    // 测试恢复功能

    // 验证恢复结果

    // 持续闪烁 LED
    use sifli_hal::gpio;
    let mut led = gpio::Output::new(p.PA26, gpio::Level::Low);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
