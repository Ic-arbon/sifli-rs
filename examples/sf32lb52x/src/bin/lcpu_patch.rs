#![no_std]
#![no_main]

//! 最小化的补丁加载测试程序。
//!
//! - 使用 HAL 的 `auto_select()` 自动识别芯片版本并安装补丁；
//! - 打印补丁条目数、通道掩码、补丁 RAM 首字；
//! - 测试保存和恢复功能；
//! - 出错时立即返回并输出 warn；

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

use sifli_hal::patch::{Patch, PatchEntry, PATCH_RAM_SYS_BASE};

#[path = "../patch_data.rs"]
mod patch_a3;
#[path = "../patch_data_rev_b.rs"]
mod patch_ls;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = sifli_hal::init(Default::default());
    let syscfg = sifli_hal::syscfg::SysCfg::new(p.HPSYS_CFG);
    let idr = syscfg.read_idr();
    let mut patch = Patch::new(p.PATCH);

    // 自动选择版本并安装
    let report = match patch
        .auto_select(
            &idr,
            &patch_a3::PATCH_RECORD_U32,
            &patch_a3::PATCH_CODE_U32,
            &patch_ls::PATCH_RECORD_U32,
            &patch_ls::PATCH_CODE_U32,
        )
        .install()
    {
        Ok(r) => r,
        Err(e) => {
            warn!("auto install failed: {:?}", e);
            return;
        }
    };

    // 读取补丁 RAM 首字（验证代码已写入）
    let first_word = unsafe { core::ptr::read_volatile(PATCH_RAM_SYS_BASE as *const u32) };

    info!(
        "Install OK: {} entries, {} channels enabled, first_word=0x{:08x}",
        report.count,
        report.mask.count(),
        first_word
    );

    // 打印启用的通道
    info!("Enabled channels:");
    for ch in report.mask.enabled_channels() {
        info!("  - Channel {}", ch);
    }

    // 测试保存功能
    let mut saved = [PatchEntry::default(); 32];
    let (count, saved_mask) = match patch.save(&mut saved) {
        Ok(result) => result,
        Err(e) => {
            warn!("save failed: {:?}", e);
            return;
        }
    };

    info!(
        "Saved: {} entries, mask={:?}, cer={:?}",
        count,
        saved_mask,
        patch.channel_mask()
    );

    // 测试恢复功能
    patch.disable_all();
    info!("Disabled all channels, cer={:?}", patch.channel_mask());

    let restore_report = match patch
        .with_entries(&saved[..count])
        .with_mask(saved_mask)
        .install()
    {
        Ok(r) => r,
        Err(e) => {
            warn!("restore failed: {:?}", e);
            return;
        }
    };

    info!(
        "Restored: {} entries, {} channels, cer={:?}",
        restore_report.count,
        restore_report.mask.count(),
        patch.channel_mask()
    );

    // 验证恢复结果
    if restore_report.mask == saved_mask {
        info!("Restore verification PASSED");
    } else {
        warn!(
            "Restore verification FAILED: expected={:?}, actual={:?}",
            saved_mask, restore_report.mask
        );
    }

    loop {
        info!("alive");
        Timer::after_secs(10).await;
    }
}
