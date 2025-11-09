#![no_std]
#![no_main]

//! 最小化的补丁加载测试程序。
//!
//! - 将 SDK 中的补丁记录与补丁代码写入补丁 RAM；
//! - 调用 HAL 的 `install_from_data` 完成安装；
//! - 打印补丁条目数、通道掩码、CER 以及补丁 RAM 首字；
//! - 出错时立即返回并输出 warn；

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

use sifli_hal::gpio;
use sifli_hal::patch::{Patch, PatchEntry};

#[path = "../patch_data.rs"]
mod patch_a3;
#[path = "../patch_data_rev_b.rs"]
mod patch_ls;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = sifli_hal::init(Default::default());
    let mut patch = Patch::new(p.PATCH);
    let mut led = gpio::Output::new(p.PA26, gpio::Level::Low);

    match patch.install_auto(
        &patch_a3::PATCH_RECORD_U32,
        &patch_a3::PATCH_CODE_U32,
        &patch_ls::PATCH_RECORD_U32,
        &patch_ls::PATCH_CODE_U32,
    ) {
        Ok((patch_type, outcome)) => {
            info!(
                "patch_type={}, entries={}, mask=0x{:08x}, cer=0x{:08x}, first=0x{:08x}",
                patch_type,
                outcome.report.entry_count,
                outcome.report.applied_mask,
                patch.cer(),
                outcome.first_word
            );

            // Test save(): capture current patch entries and CER
            let mut saved: [PatchEntry; 32] = [PatchEntry::default(); 32];
            match patch.save(&mut saved) {
                Ok((count, cer)) => {
                    info!("save: count={}, cer=0x{:08x}", count, cer);
                    // Re-apply using saved table to validate semantics
                    patch.disable_all();
                    match patch.apply_with_mask(&saved[..count], cer) {
                        Ok(mask) => info!(
                            "restore: mask=0x{:08x}, cer=0x{:08x}",
                            mask,
                            patch.cer()
                        ),
                        Err(e) => warn!("restore failed: {:?}", e),
                    }
                }
                Err(e) => warn!("save failed: {:?}", e),
            }
        }
        Err(err) => {
            warn!("auto install failed: {:?}", err);
            return;
        }
    }

    loop {
        led.toggle();
        Timer::after_secs(1).await;
    }
}
