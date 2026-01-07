#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use static_cell::StaticCell;

use sifli_hal::{gpio, lcdc, rcc};
use sifli_hal::rcc::{ClkSysSel, ConfigOption, DllConfig};
use sifli_hal::bind_interrupts;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    pixelcolor::{raw::{BigEndian, RawU16}, Rgb565},
    prelude::*,
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    text::Text,
};

// Import the display driver modules
use display_driver::ColorFormat;
use display_driver::display_bus::QspiFlashBus;
use dd_co5300::{Co5300, spec::DisplaySpec};
use display_driver::panel::{LCDResetOption, Panel};

// --- Constants & Types ---

const WIDTH: usize = 240;
const HEIGHT: usize = 240;

/// Board-specific display specification
pub struct MyCo5300;
impl DisplaySpec for MyCo5300 {
    const WIDTH: u16 = WIDTH as u16;
    const HEIGHT: u16 = HEIGHT as u16;
    const COL_OFFSET: u16 = 0;
    const ROW_OFFSET: u16 = 0;
    const INIT_PAGE_PARAM: u8 = 0x20; 
    const IGNORE_ID_CHECK: bool = false;
}

// Framebuffer configuration
// Using BigEndian for direct compatibility with the display controller's byte order
type FramebufferType = Framebuffer<
    Rgb565, 
    RawU16, 
    BigEndian, 
    WIDTH, 
    HEIGHT, 
    { buffer_size::<Rgb565>(WIDTH, HEIGHT) }
>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

const IMAGE_WIDTH: u32 = 86;
const IMAGE_HEIGHT: u32 = 64;

bind_interrupts!(
    struct Irqs {
        LCDC1 => sifli_hal::lcdc::InterruptHandler<sifli_hal::peripherals::LCDC1>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Init SF32LB52 @ 240MHz...");

    // 1. Hardware Initialization
    let mut config = sifli_hal::Config::default();
    // 240MHz Dll1 Freq = (stg + 1) * 24MHz -> (9 + 1) * 24 = 240
    config.rcc.dll1 = ConfigOption::Update(DllConfig { enable: true, stg: 9, div2: false });
    config.rcc.clk_sys_sel = ConfigOption::Update(ClkSysSel::Dll1);
    
    let p = sifli_hal::init(config);
    rcc::test_print_clocks();

    // 2. LCDC Configuration
    let config = sifli_hal::lcdc::Config { 
        width: WIDTH as u16, 
        height: HEIGHT as u16,
        ..Default::default() 
    };
    
    let mut lcdc = lcdc::Lcdc::new_qspi(
        p.LCDC1, Irqs,
        p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA8,
        config
    );
    lcdc.init();
    
    // Wrap the raw bus in the QspiFlashBus protocol layer (handles 0x02/0x32 prefixes)
    let mut disp_bus = QspiFlashBus::new(lcdc);

    let rst = gpio::Output::new(p.PA0, gpio::Level::Low);
    let mut bl = gpio::Output::new(p.PA1, gpio::Level::Low);

    // Initialize the CO5300 panel driver
    let mut panel = Co5300::<MyCo5300, _, _>::new(LCDResetOption::new_pin(rst));


    info!("Initializing Display...");
    // The panel init sequence handles reset and configuration
    panel.init(&mut disp_bus, &mut Delay).await.unwrap();
    panel.set_color_format(&mut disp_bus, ColorFormat::RGB565).await.unwrap();
    panel.set_brightness(&mut disp_bus, 255).await.unwrap();

    // Enable backlight
    bl.set_low();

    // 4. Graphics Setup
    let fb = FB.init(Framebuffer::new());
    fb.clear(Rgb565::WHITE).unwrap();

    // Load and draw image
    let image_raw: ImageRaw<Rgb565, BigEndian> = ImageRaw::new(
        include_bytes!("../../assets/ferris.raw"), 
        IMAGE_WIDTH
    );

    let image = Image::new(
        &image_raw, 
        Point::new(
            ((WIDTH as i32) - (IMAGE_WIDTH as i32)) / 2,
            ((HEIGHT as i32) - (IMAGE_HEIGHT as i32)) / 2,
        )
    );
    image.draw(fb).unwrap();

    // Draw text
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
    Text::new("Hello SF32!", Point::new(50, 50), style)
        .draw(fb)
        .unwrap();

    Text::new("SiFli-rs!", Point::new(50, 80), style)
        .draw(fb)
        .unwrap();

    info!("Starting render loop...");

    // 5. Render Loop
    loop {
        info!("Sending Frame ...");
        // Update the display with the new frame
        panel.write_pixels(
            &mut disp_bus,
            0, 
            0, 
            WIDTH as u16 - 1, 
            HEIGHT as u16 - 1, 
            fb.data()
        ).await.unwrap();

        info!("Frame Finished");
        Timer::after_secs(5).await;
    }
}