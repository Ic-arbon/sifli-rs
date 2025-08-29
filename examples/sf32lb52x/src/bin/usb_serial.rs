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

use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use static_cell::StaticCell;
use embassy_executor::Spawner;

use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::UsbDevice;

use sifli_hal::bind_interrupts;
use sifli_hal::rcc::{ClkSysSel, ConfigOption, DllConfig, UsbConfig, UsbSel};
use sifli_hal::usb::{Driver, InterruptHandler, Instance};


bind_interrupts!(struct Irqs {
    USBC => InterruptHandler<sifli_hal::peripherals::USBC>;
});

// you can use `arch-spin` instead of `arch-cortex-m` in embassy-executor's
// feature by setting `entry="cortex_m_rt::entry"`.
// This Will NOT enter Wfi during executor idle.
#[embassy_executor::main(entry="cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
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
    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("SiFli-rs");
        config.product = Some("sifli-rs USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Required for windows compatibility.
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;
        config
    };

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    // Create classes on the builder.
    let mut class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    // Build the builder.
    let usb = builder.build();

    // Run the USB device.
    unwrap!(spawner.spawn(usb_task(usb)));

    // Do stuff with the class!
    loop {
        class.wait_connection().await;
        info!("Connected");
        let _ = echo(&mut class).await;
        info!("Disconnected");
    }
}

type MyUsbDriver = Driver<'static, sifli_hal::peripherals::USBC>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

#[embassy_executor::task]
async fn usb_task(mut usb: MyUsbDevice) -> ! {
    usb.run().await
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => defmt::panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
