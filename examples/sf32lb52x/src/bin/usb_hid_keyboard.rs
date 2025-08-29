//! USB HID keyboard example
//!
//! For some computers/hosts, power on first and wait for the bootloader to finish 
//! (at least 3s) before plugging in the USB cable.  
//! Some hosts may misidentify the chip running the bootloader as a USB device 
//! (even though the PHY is not enabled) and try enumeration. 
//! After multiple failures, they stop retrying, causing the device to be unrecognized.  
//! The same issue exists in SiFli-SDK examples.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_futures::join::join;

use embassy_usb::class::hid::{HidReaderWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Handler};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use sifli_hal::bind_interrupts;
use sifli_hal::rcc::{ClkSysSel, ConfigOption, DllConfig, UsbConfig, UsbSel};
use sifli_hal::usb::{Driver, InterruptHandler};

bind_interrupts!(struct Irqs {
    USBC => InterruptHandler<sifli_hal::peripherals::USBC>;
});

// you can use `arch-spin` instead of `arch-cortex-m` in embassy-executor's
// feature by setting `entry="cortex_m_rt::entry"`.
// This Will NOT enter Wfi during executor idle.
#[embassy_executor::main(entry="cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello World! USB HID TEST");
    let mut config = sifli_hal::Config::default();
    // 240MHz Dll1 Freq = (stg + 1) * 24MHz
    config.rcc.dll1 = ConfigOption::Update(DllConfig { enable: true, stg: 9, div2: false });
    config.rcc.clk_sys_sel = ConfigOption::Update(ClkSysSel::Dll1);
    config.rcc.usb = ConfigOption::Update(UsbConfig { sel: UsbSel::ClkSys, div: 4 });
    let p = sifli_hal::init(config);

    sifli_hal::rcc::test_print_clocks();

    // Create the driver, from the HAL
    let driver = Driver::new(p.USBC, Irqs, p.PA35, p.PA36);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("SiFli-rs");
    config.product = Some("HID keyboard example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    // You can also add a Microsoft OS descriptor.
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    builder.handler(&mut device_handler);

    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 60,
        max_packet_size: 8,
    };

    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let (reader, mut writer) = hid.split();

    // let mut button = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up);
    
    // Do stuff with the class!
    let in_fut = async {
        loop {
            embassy_time::Timer::after_secs(1).await;
            // button.wait_for_falling_edge().await;
            info!("Button pressed!");
            // Create a report with the A key pressed. (no shift modifier)
            let report = KeyboardReport {
                keycodes: [4, 0, 0, 0, 0, 0],
                leds: 0,
                modifier: 0,
                reserved: 0,
            };
            // Send the report.
            match writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            };

            embassy_time::Timer::after_millis(300).await;
            // button.wait_for_rising_edge().await;
            info!("Button released!");
            let report = KeyboardReport {
                keycodes: [0, 0, 0, 0, 0, 0],
                leds: 0,
                modifier: 0,
                reserved: 0,
            };
            match writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            };
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(in_fut, out_fut)).await;
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
