#![no_std]
#![no_main]

//! LCPU 镜像安装示例。

use defmt::{info, error};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[path = "../lcpu_image_52x.rs"]
mod lcpu_image_52x;

/// LCPU 固件镜像（通过 `contrib/carray2rs` 从 SDK C 数组生成）
///
/// 重新生成命令：
/// ```sh
/// cd sifli-rs/contrib/carray2rs
/// cargo run -- ../../../../SiFli-SDK/example/rom_bin/lcpu_general_ble_img/lcpu_52x.c > ../../examples/sf32lb52x/src/lcpu_image_52x.rs
/// ```
const LCPU_IMAGE: &[u32] = &lcpu_image_52x::G_LCPU_BIN_U32;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = sifli_hal::init(Default::default());
    let syscfg_idr = sifli_hal::syscfg::SysCfg::new(_p.HPSYS_CFG).read_idr();

    info!("LCPU image installation example");

    match sifli_hal::lcpu::install(&syscfg_idr,LCPU_IMAGE) {
        Ok(()) => info!("Success"),
        Err(err) => error!("Failed: {:?}", err),
    }

    loop {
        Timer::after_secs(1).await;
    }
}
