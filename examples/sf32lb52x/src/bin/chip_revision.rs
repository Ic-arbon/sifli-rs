#![no_std]
#![no_main]

//! 读取并显示芯片版本信息（Revision ID）。
//!
//! 本示例演示如何使用 sifli-hal 的 syscfg 模块读取芯片标识信息。

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;
use sifli_hal::syscfg::{ChipRevision, PatchType, SysCfg};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = sifli_hal::init(Default::default());

    info!("========================================");
    info!("SF32LB52x 芯片版本检测");
    info!("========================================");

    // 读取 IDR 寄存器（无需外设所有权）
    let idr = SysCfg::read_idr();
    let revision = idr.revision();

    info!("IDR 寄存器: {:?}", idr);
    info!("");
    info!("详细信息:");
    info!("  - Revision ID: 0x{:02x} ({})", idr.revid, revision.name());
    info!("  - Package ID:  0x{:02x}", idr.pid);
    info!("  - Company ID:  0x{:02x}", idr.cid);
    info!("  - Series ID:   0x{:02x}", idr.sid);
    info!("");
    info!("芯片版本: {:?}", revision);
    info!("SDK 有效性检查: {}", revision.is_valid());

    // 根据版本给出建议
    if !revision.is_valid() {
        info!("");
        info!("警告: 无效的芯片版本！");
        info!("  - REVID 0x{:02x} 不在SDK支持范围内", idr.revid);
        info!("  - 请联系厂商确认版本信息");
    } else {
        info!("");
        match revision.patch_type() {
            Some(PatchType::A3) => {
                info!("补丁类型: {:?}", PatchType::A3);
                info!("提示:");
                match revision {
                    ChipRevision::A3OrEarlier(0x00..=0x02) => {
                        info!("  - 早期工程样片（ES）版本");
                        info!("  - 使用 lcpu_patch.c 中的 A3 补丁");
                    }
                    ChipRevision::A3OrEarlier(0x03) => {
                        info!("  - A3 量产版本");
                        info!("  - 使用 lcpu_patch.c 中的 A3 补丁");
                    }
                    _ => {}
                }
                info!("  - 需要从 Flash 加载 LCPU 镜像");
            }
            Some(PatchType::LetterSeries) => {
                info!("补丁类型: {:?}", PatchType::LetterSeries);
                info!("提示:");
                match revision {
                    ChipRevision::A4 => {
                        info!("  - A4 版本芯片（Letter Series）");
                    }
                    ChipRevision::B4 => {
                        info!("  - B4 版本芯片（Letter Series 最新版）");
                    }
                    _ => {}
                }
                info!("  - 使用 lcpu_patch_rev_b.c 中的新补丁");
                info!("  - LCPU 可直接从 ROM 运行");
            }
            None => {
                info!("补丁类型: 无法确定");
            }
        }
        info!("");
        info!("Letter Series: {}", revision.is_letter_series());
    }

    info!("========================================");

    // 持续闪烁LED表示程序运行正常
    use sifli_hal::gpio;
    let mut led = gpio::Output::new(p.PA26, gpio::Level::Low);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
