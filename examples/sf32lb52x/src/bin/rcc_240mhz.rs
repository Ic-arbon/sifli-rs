#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;
use embassy_time::Timer;
use embassy_executor::Spawner;

use sifli_hal;
use sifli_hal::gpio;
use sifli_hal::rcc::{self, Dll, DllStage, Sysclk};

// **WARN**:
// The RCC clock configuration module is still under construction, 
// and there is no guarantee that other clock configurations will 
// run correctly.
// https://github.com/OpenSiFli/sifli-rs/issues/7

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");
    let mut config = sifli_hal::Config::default();
    
    // Configure 240MHz system clock using DLL1
    // DLL1 Freq = (stg + 1) * 24MHz = (9 + 1) * 24MHz = 240MHz
    config.rcc.sys = Sysclk::DLL1;
    config.rcc.dll1 = Some(Dll {
        out_div2: false,
        stg: DllStage::MUL10,  // MUL10 = enum value 9
    });
    
    let p = sifli_hal::init(config);

    info!("Clock configuration complete");
    rcc::test_print_clocks();

    // SF32LB52-DevKit-LCD LED pin
    let mut led = gpio::Output::new(p.PA1, gpio::Level::Low);
    
    loop {
        info!("led on!");
        led.set_high();
        Timer::after_secs(1).await;

        info!("led off!");
        led.set_low();
        Timer::after_secs(1).await;
    }
}
