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
use sifli_hal::patch::Patch;

#[path = "../patch_data.rs"]
mod patch_data;

use patch_data::{PATCH_CODE_U32, PATCH_RECORD_U32};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = sifli_hal::init(Default::default());
    let mut patch = Patch::new(p.PATCH);
    let mut led = gpio::Output::new(p.PA26, gpio::Level::Low);

    match patch.install_from_data(&PATCH_RECORD_U32, &PATCH_CODE_U32) {
        Ok(outcome) => info!(
            "entries={}, mask=0x{:08x}, cer=0x{:08x}, first=0x{:08x}",
            outcome.report.entry_count,
            outcome.report.applied_mask,
            patch.cer(),
            outcome.first_word
        ),
        Err(err) => {
            warn!("install failed: {:?}", err);
            return;
        }
    }

    loop {
        led.toggle();
        Timer::after_secs(1).await;
    }
}
