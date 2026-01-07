use display_driver::{DisplayError, display_bus};
use display_driver::display_bus::DisplayBus;
use embassy_hal_internal::into_ref;
use embassy_time::{Duration, Instant, Timer};

use crate::gpio::{AfType, Pull};
use crate::pac::lcdc::vals;
use crate::rcc::enable_and_reset;
use crate::time::Hertz;
use crate::to_system_bus_addr;
use crate::utils::blocking_wait_timeout_ms;
use crate::{interrupt, peripherals, Peripheral};

pub use vals::{
    AlphaSel, LayerFormat, LcdFormat, LcdIntfSel, Polarity, SingleAccessType, SpiAccessLen,
    SpiClkInit, SpiClkPol, SpiLcdFormat, SpiLineMode, SpiRdMode, TargetLcd,
};
pub use vals::LcdIntfSel as LcdInterface;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    Spi,
    // MCU 8080
    Dbi,
    // MIPI
    Dsi,
    // RGB
    Dpi,
    // JDI
    Jdi,
}

pub const INTERFACE_COLOR_FORMATS: &[(InterfaceType, &[OutputColorFormat])] = &[
    (
        InterfaceType::Spi,
        &[
            OutputColorFormat::Rgb332,
            OutputColorFormat::Rgb565,
            // OutputColorFormat::Rgb565Swap,
            OutputColorFormat::Rgb888,
        ],
    ),
    (
        InterfaceType::Dbi,
        &[
            OutputColorFormat::Rgb332,
            OutputColorFormat::Rgb565,
            // OutputColorFormat::Rgb565Swap,
            OutputColorFormat::Rgb888,
        ],
    ),
];

impl InterfaceType {
    pub fn from_lcd_interface(intf: LcdIntfSel) -> Self {
        match intf {
            LcdIntfSel::Spi => InterfaceType::Spi,
            LcdIntfSel::DbiTypeB => InterfaceType::Dbi,
            LcdIntfSel::DbiToDsi => InterfaceType::Dsi,
            LcdIntfSel::Dpi => InterfaceType::Dpi,
            LcdIntfSel::JdiSerial => InterfaceType::Jdi,
            LcdIntfSel::JdiParallel => InterfaceType::Jdi,
            LcdIntfSel::DbiTypeA => InterfaceType::Dbi,
            LcdIntfSel::DpiToDsi => InterfaceType::Dsi,
        }
    }

    pub fn supports_output_format(&self, format: &OutputColorFormat) -> bool {
        INTERFACE_COLOR_FORMATS
            .iter()
            .find(|(intf, _)| intf == self)
            .is_some_and(|(_, formats)| formats.contains(format))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputColorFormat {
    Rgb332,
    Rgb565,
    // Rgb565Swap,
    Rgb888,
    Argb8565,
    Argb8888,
    /// Luminance 8-bit
    A8,
    /// Alpha 8-bit
    #[cfg(not(feature = "sf32lb52x"))]
    L8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputColorFormat {
    Rgb332,
    Rgb565,
    // Rgb565Swap,
    /// DSI Only
    Rgb666,
    Rgb888,
}

impl InputColorFormat {
    pub fn to_layer_format(&self) -> LayerFormat {
        match self {
            InputColorFormat::Rgb332 => LayerFormat::RGB332,
            InputColorFormat::Rgb565 => LayerFormat::RGB565,
            // InputColorFormat::Rgb565Swap => LayerFormat::RGB565,
            InputColorFormat::Rgb888 => LayerFormat::RGB888,
            InputColorFormat::Argb8565 => LayerFormat::ARGB8565,
            InputColorFormat::Argb8888 => LayerFormat::ARGB8888,
            InputColorFormat::A8 => LayerFormat::A8,
            #[cfg(not(feature = "sf32lb52x"))]
            InputColorFormat::L8 => LayerFormat::L8,
        }
    }

    pub fn bpp(&self) -> u16 {
        match self {
            InputColorFormat::Rgb332 => 1,
            InputColorFormat::Rgb565 => 2,
            // InputColorFormat::Rgb565Swap => 2,
            InputColorFormat::Rgb888 => 3,
            InputColorFormat::Argb8565 => 3,
            InputColorFormat::Argb8888 => 4,
            InputColorFormat::A8 => 1,
            #[cfg(not(feature = "sf32lb52x"))]
            InputColorFormat::L8 => 1,
        }
    }
}

impl OutputColorFormat {
    pub fn to_lcd_format(&self, interface: InterfaceType) -> LcdFormat {
        assert!(interface == InterfaceType::Spi);
        match self {
            OutputColorFormat::Rgb332 => LcdFormat::Rgb332,
            OutputColorFormat::Rgb565 => LcdFormat::Rgb565,
            // OutputColorFormat::Rgb565Swap => LcdFormat::Rgb565,
            OutputColorFormat::Rgb888 => LcdFormat::Rgb888,
            _ => panic!(),
        }
    }

    pub fn to_spi_lcd_format(&self) -> SpiLcdFormat {
        match self {
            OutputColorFormat::Rgb332 => SpiLcdFormat::Rgb332,
            OutputColorFormat::Rgb565 => SpiLcdFormat::Rgb565,
            // OutputColorFormat::Rgb565Swap => SpiLcdFormat::Rgb565,
            OutputColorFormat::Rgb888 => SpiLcdFormat::Rgb888,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrequencyConfig {
    /// Use fixed frequency
    Freq(Hertz),
    /// Use source clock divided by this value (minimum divider is 2)
    Div(u8),
}

/// SPI Configuration for the LCD interface
#[derive(Debug, Clone)]
pub struct SpiConfig {
    /// SPI line mode (e.g., 4-line, 3-line, etc.)
    pub line_mode: SpiLineMode,
    /// SPI clock polarity (CPOL)
    pub clk_polarity: SpiClkPol,
    /// SPI clock phase (CPHA)
    pub clk_phase: SpiClkInit,
    /// Chip select polarity
    pub cs_polarity: Polarity,
    /// VSYNC signal polarity (used for TE)
    pub vsyn_polarity: Polarity,
    /// SPI write frequency
    pub write_frequency: FrequencyConfig,
    /// SPI read frequency
    pub read_frequency: FrequencyConfig,
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            line_mode: SpiLineMode::FourLine4Data,
            clk_polarity: SpiClkPol::Normal,
            clk_phase: SpiClkInit::Low,
            cs_polarity: Polarity::ActiveLow,
            vsyn_polarity: Polarity::ActiveLow,
            write_frequency: FrequencyConfig::Freq(Hertz::mhz(10)), // 10 MHz
            read_frequency: FrequencyConfig::Freq(Hertz::mhz(2)),   // 2 MHz
        }
    }
}

/// Main configuration for the LCDC driver
#[derive(Debug, Clone)]
pub struct Config {
    pub width: u16,
    pub height: u16,

    /// Output pixel format (target format sent to the display)
    pub out_color_format: OutputColorFormat,
    /// Input layer format (format of the framebuffer in memory)
    pub in_color_format: InputColorFormat,
    /// LCD reset interval in microseconds
    pub reset_lcd_interval_us: u32,
    /// TODO: Display interface selection (SPI/DBI/DSI)
    pub display_interface: LcdIntfSel,
    /// SPI specific settings
    pub spi: SpiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 240,
            height: 240,
            out_color_format: OutputColorFormat::Rgb565,
            in_color_format: InputColorFormat::Rgb565,
            display_interface: LcdIntfSel::Spi,
            reset_lcd_interval_us: 20,
            spi: SpiConfig::default(),
        }
    }
}

/// LCDC Driver implementation for SF32LB52x
pub struct Lcdc<'d, T: Instance> {
    _peri: crate::PeripheralRef<'d, T>,
    config: Config,
}

impl<'d, T: Instance> Lcdc<'d, T> {
    /// Create a new LCDC QSPI driver instance
    pub fn new_qspi(peri: impl Peripheral<P = T> + 'd,
        spi_te: impl Peripheral<P = impl SpiTePin<T>> + 'd,
        spi_cs: impl Peripheral<P = impl SpiCsPin<T>> + 'd,
        spi_clk: impl Peripheral<P = impl SpiClkPin<T>> + 'd,
        spi_dio0: impl Peripheral<P = impl SpiDio0Pin<T>> + 'd,
        spi_dio1: impl Peripheral<P = impl SpiDio1Pin<T>> + 'd,
        spi_dio2: impl Peripheral<P = impl SpiDio2Pin<T>> + 'd,
        spi_dio3: impl Peripheral<P = impl SpiDio3Pin<T>> + 'd,
        config: Config
    ) -> Self {
        into_ref!(peri);
        init_pin!(spi_te, AfType::new(Pull::None));
        init_pin!(spi_cs, AfType::new(Pull::None));
        init_pin!(spi_clk, AfType::new(Pull::None));
        init_pin!(spi_dio0, AfType::new(Pull::None));
        init_pin!(spi_dio1, AfType::new(Pull::None));
        init_pin!(spi_dio2, AfType::new(Pull::None));
        init_pin!(spi_dio3, AfType::new(Pull::None));

        assert!(
            config.display_interface == LcdIntfSel::Spi,
            "Only SPI interface is supported now"
        );
        assert!(
            InterfaceType::from_lcd_interface(config.display_interface)
                .supports_output_format(&config.out_color_format),
            "Unsupported output format for SPI interface"
        );

        Self {
            _peri: peri,
            config,
        }
    }

    /// TODO
    pub fn new_qspi_with_rstb(peri: impl Peripheral<P = T> + 'd,
        spi_rstb: impl Peripheral<P = impl SpiRstbPin<T>> + 'd,
        spi_te: impl Peripheral<P = impl SpiTePin<T>> + 'd,
        spi_cs: impl Peripheral<P = impl SpiCsPin<T>> + 'd,
        spi_clk: impl Peripheral<P = impl SpiClkPin<T>> + 'd,
        spi_dio0: impl Peripheral<P = impl SpiDio0Pin<T>> + 'd,
        spi_dio1: impl Peripheral<P = impl SpiDio1Pin<T>> + 'd,
        spi_dio2: impl Peripheral<P = impl SpiDio2Pin<T>> + 'd,
        spi_dio3: impl Peripheral<P = impl SpiDio3Pin<T>> + 'd,
        config: Config
    ) -> Self {
        init_pin!(spi_rstb, AfType::new(Pull::Down));
        Self::new_qspi(peri, spi_te, spi_cs, spi_clk, spi_dio0, spi_dio1, spi_dio2, spi_dio3, config)
    }

    pub fn set_spi_frequency(&mut self, freq: FrequencyConfig) {
        // Calculate clock divider
        // Formula: clk_div = (src_clk + freq - 1) / freq
        // Hardware requirement: divider >= 2
        let regs = T::regs();
        let clk_div = match freq {
            FrequencyConfig::Freq(hz) => {
                if hz.0 > 0 {
                    T::get_freq().unwrap().0.div_ceil(hz.0)
                } else {
                    2
                }
            }
            FrequencyConfig::Div(div) => div as u32,
        }
        .max(2) as u8;
        regs.spi_if_conf().modify(|w| w.set_clk_div(clk_div));
    }

    /// Reset the LCD via the dedicated reset pin (hardware reset).
    /// This matches C code `HAL_LCDC_ResetLCD` which toggles `LCD_RSTB` pin.
    pub async fn reset_lcd(&mut self) {
        let regs = T::regs();

        // Assert reset (Active Low)
        regs.lcd_if_conf().modify(|w| w.set_lcd_rstb(false));
        Timer::after(Duration::from_micros(
            self.config.reset_lcd_interval_us as u64,
        ))
        .await;
        // Release reset
        regs.lcd_if_conf().modify(|w| w.set_lcd_rstb(true));
    }

    /// Initialize the LCDC peripheral
    pub fn init(&mut self) {
        let regs = T::regs();
        enable_and_reset::<T>();

        regs.setting().modify(|w| w.set_auto_gate_en(true));

        let lcd_format =
            self.config
                .out_color_format
                .to_lcd_format(InterfaceType::from_lcd_interface(
                    self.config.display_interface,
                ));

        let spi_lcd_format = self.config.out_color_format.to_spi_lcd_format();

        // Configure LCD Interface (LCD_CONF)
        regs.lcd_conf().modify(|w| {
            w.set_lcd_intf_sel(LcdIntfSel::Spi);
            w.set_target_lcd(TargetLcd::LcdPanel0);
            w.set_lcd_format(lcd_format);

            w.set_spi_lcd_format(spi_lcd_format);
        });

        // Configure SPI Interface (SPI_IF_CONF)
        regs.spi_if_conf().modify(|w| {
            w.set_line(self.config.spi.line_mode);
            w.set_spi_cs_pol(self.config.spi.cs_polarity);
            w.set_spi_clk_pol(self.config.spi.clk_polarity);
            w.set_spi_clk_init(self.config.spi.clk_phase);
            w.set_spi_clk_auto_dis(true); // Disable CLK when idle
            w.set_spi_cs_no_idle(true); // Keep CS active during transaction
            w.set_dummy_cycle(0);
        });
        self.set_spi_frequency(self.config.spi.write_frequency);

        // Configure Tearing Effect (TE) - Disabled for now
        regs.te_conf().write(|w| w.set_enable(false));

        // Release the LCD reset signal (Active Low usually, set to 1 to release)
        regs.lcd_if_conf().modify(|w| w.set_lcd_rstb(true));
    }

    /// Helper: Wait for the Single Access (Command/Param) interface to be ready.
    /// Uses a blocking wait with timeout since command transmission is very fast.
    fn wait_single_busy(&self) -> Result<(), Error> {
        let regs = T::regs();

        blocking_wait_timeout_ms(|| regs.lcd_single().read().lcd_busy(), 100)
            .map_err(|_| Error::Timeout)
    }

    /// Helper: Wait for the interface to be ready.
    /// Checks STATUS.lcd_busy and LCD_SINGLE.lcd_busy.
    fn wait_busy(&self) -> Result<(), Error> {
        let regs = T::regs();
        blocking_wait_timeout_ms(
            || regs.status().read().lcd_busy() || regs.lcd_single().read().lcd_busy(),
            100,
        )
        .map_err(|_| Error::Timeout)
    }

    /// Send a command to the LCD via SPI.
    ///
    /// This is blocking because SPI commands are short.
    /// Checks full busy status before starting.
    pub fn send_cmd(&mut self, cmd: u32, len_bytes: u8, continuous: bool) -> Result<(), Error> {
        if len_bytes == 0 || len_bytes > 4 {
            return Err(Error::InvalidParameter);
        }

        self.wait_busy()?;

        let regs = T::regs();

        regs.spi_if_conf().modify(|w| {
            // Set write mode to normal
            w.set_spi_rd_mode(SpiRdMode::Normal);
            w.set_spi_cs_auto_dis(!continuous);

            let len_val = match len_bytes {
                1 => SpiAccessLen::Bytes1, // SpiAccessLen::Bytes1 == 0
                2 => SpiAccessLen::Bytes2,
                3 => SpiAccessLen::Bytes3,
                4 => SpiAccessLen::Bytes4,
                _ => unreachable!(),
            };
            w.set_wr_len(len_val);
        });

        // Write the command data
        regs.lcd_wr().write(|w| w.set_data(cmd));

        // Trigger the transfer as a Command
        regs.lcd_single().write(|w| {
            w.set_wr_trig(true);
            w.set_type_(SingleAccessType::Command);
        });

        Ok(())
    }

    /// Send data parameter to the LCD via SPI.
    ///
    /// This is blocking, used for sending parameters immediately after a command.
    pub fn send_cmd_data(&mut self, data: u32, len_bytes: u8, continuous: bool) -> Result<(), Error> {
        if len_bytes == 0 || len_bytes > 4 {
            return Err(Error::InvalidParameter);
        }

        self.wait_single_busy()?;

        let regs = T::regs();

        regs.spi_if_conf().modify(|w| {
            w.set_spi_cs_auto_dis(!continuous);
            let len_val = match len_bytes {
                1 => SpiAccessLen::Bytes1,
                2 => SpiAccessLen::Bytes2,
                3 => SpiAccessLen::Bytes3,
                4 => SpiAccessLen::Bytes4,
                _ => unreachable!(),
            };
            w.set_wr_len(len_val);
        });

        regs.lcd_wr().write(|w| w.set_data(data));

        // Trigger the transfer as Data
        regs.lcd_single().write(|w| {
            w.set_wr_trig(true);
            w.set_type_(SingleAccessType::Data);
        });

        Ok(())
    }

    /// Send pixel data (framebuffer) asynchronously.
    ///
    /// use #[repr(align(4))] to your buffer to ensure proper alignment.
    ///
    /// The signature is `async`, but the current implementation uses a polled wait (dead wait)
    /// for the End-Of-Frame (EOF) flag to allow for fast verification without complex interrupt handling.
    pub async fn send_pixel_data(
        &mut self,
        buffer: &[u8],
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), Error> {
        debug_assert!(
            buffer.as_ptr() as usize % 4 == 0,
            "Buffer address must be 4-byte aligned"
        );

        let regs = T::regs();

        // Ensure previous operations are complete
        self.wait_busy()?;

        let width = x1 - x0 + 1;
        // let height = y1 - y0 + 1; // Unused for now

        // Configure Canvas Area (ROI)
        regs.canvas_tl_pos().write(|w| {
            w.set_x0(x0);
            w.set_y0(y0);
        });
        regs.canvas_br_pos().write(|w| {
            w.set_x1(x1);
            w.set_y1(y1);
        });

        let layer_format = self.config.in_color_format.to_layer_format();

        // Configure Layer 0
        regs.layer0_config().write(|w| {
            w.set_active(true);
            w.set_format(layer_format);
            w.set_alpha(255); // Fully opaque
            w.set_alpha_sel(AlphaSel::Layer);
            w.set_prefetch_en(true);
            w.set_v_mirror(false);

            // Calculate width in bytes for the pitch
            let bpp = self.config.in_color_format.bpp();
            let line_width_bytes = width * bpp;
            w.set_width(line_width_bytes);
        });

        regs.layer0_tl_pos().write(|w| {
            w.set_x0(x0);
            w.set_y0(y0);
        });
        regs.layer0_br_pos().write(|w| {
            w.set_x1(x1);
            w.set_y1(y1);
        });

        regs.spi_if_conf().modify(|w| {
            w.set_spi_cs_auto_dis(true);
        });

        // Set Source Address
        // Note: Buffer alignment requirements
        let addr = to_system_bus_addr(buffer.as_ptr() as usize) as u32;
        regs.layer0_src().write(|w| w.set_addr(addr));

        // Start Transfer
        regs.command().write(|w| w.set_start(true));

        // Wait for transfer completion (EOF)
        // Using a loop with a small async sleep to allow the executor to do other things,
        // effectively polling the hardware register.
        let start_wait = Instant::now();
        loop {
            let irq = regs.irq().read();

            // Check for HW errors
            if irq.dpi_udr_raw_stat() || irq.icb_of_raw_stat() {
                // Clear error flags
                regs.irq().write(|w| {
                    w.set_dpi_udr_stat(true);
                    w.set_icb_of_stat(true);
                });
                return Err(Error::HardwareError(irq.0));
            }

            // Check for End Of Frame
            if irq.eof_raw_stat() {
                // Clear EOF flag
                regs.irq().write(|w| w.set_eof_stat(true));
                break;
            }

            if start_wait.elapsed() > Duration::from_millis(10000) {
                return Err(Error::Timeout);
            }

            // Yield briefly
            Timer::after(Duration::from_micros(50)).await;
        }

        Ok(())
    }

    pub async fn send_pixel_data_framebuffer(
        &mut self,
        buffer: &[u8],
    ) -> Result<(), Error> {
        self.send_pixel_data_rect(self.config.width, self.config.height, buffer).await
    }

    pub async fn send_pixel_data_rect(
        &mut self,
        width: u16,
        height: u16,
        buffer: &[u8],
    ) -> Result<(), Error> {
        let bpp = self.config.in_color_format.bpp() as usize;
        let len = buffer.len();
        
        if len % bpp != 0 {
            return Err(Error::UnalignedData);
        }
        
        let pixel_count = len / bpp;

        if (width as usize) * (height as usize) != pixel_count {
            error!("Buffer size mismatch: expected {} bytes, got {} (width={}, height={})", (width as usize) * (height as usize) * bpp, len, width, height);
            return Err(Error::UnalignedData);
        }

        self.send_pixel_data(
            buffer,
            0,
            0,
            width - 1,
            height - 1,
        )
        .await
    }
}

/// Errors that can occur during LCD operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Timeout,
    InvalidParameter,
    UnalignedData,
    HardwareError(u32),

}

// ============================================================================
// Trait Definitions
// ============================================================================

pin_trait!(SpiRstbPin, Instance);
pin_trait!(SpiTePin, Instance);
pin_trait!(SpiCsPin, Instance);
pin_trait!(SpiClkPin, Instance);
pin_trait!(SpiDio0Pin, Instance);
pin_trait!(SpiDio1Pin, Instance);
pin_trait!(SpiDio2Pin, Instance);
pin_trait!(SpiDio3Pin, Instance);


pub(crate) trait SealedInstance:
    crate::rcc::RccEnableReset + crate::rcc::RccGetFreq
{
    fn regs() -> crate::pac::lcdc::Lcdc;
}

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl SealedInstance for peripherals::LCDC1 {
    fn regs() -> crate::pac::lcdc::Lcdc {
        crate::pac::LCDC1
    }
}
impl Instance for peripherals::LCDC1 {
    type Interrupt = crate::interrupt::typelevel::LCDC1;
}

// ============================================================================
// Trait Implementations
// ============================================================================


impl<'d, T: Instance> DisplayBus for Lcdc<'d, T> {
    type Error = Error;
    
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        if cmd.is_empty() || cmd.len() > 4 {
            return Err(Error::InvalidParameter);
        }
        
        let cmd_word = cmd.iter()
            .fold(0u32, |acc, &byte| (acc << 8) | (byte as u32));
        
        self.send_cmd(cmd_word, cmd.len() as u8, false)
    }
    
    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        if cmd.is_empty() || cmd.len() > 4 {
            return Err(Error::InvalidParameter);
        }
        
        let cmd_word = cmd.iter()
            .fold(0u32, |acc, &byte| (acc << 8) | (byte as u32));
        
        self.send_cmd(cmd_word, cmd.len() as u8, params.len() != 0)?;

        if params.len() > 0 {
            params.chunks(4)
            .enumerate()
            .try_for_each(|(i, chunk)| {
                let data_word = chunk
                    .iter()
                    .fold(0u32, |acc, &byte| (acc << 8) | (byte as u32));
                
                let is_last = (i + 1) * 4 >= params.len();
                self.send_cmd_data(data_word, chunk.len() as u8, !is_last)
            })
        } else {
            Ok(())
        }
    }
    
    async fn write_pixels(&mut self, cmd: &[u8], params: &[u8], buffer: &[u8], metadata: display_bus::Metadata) -> Result<(), DisplayError<Self::Error>> {
        if params.len() > 0 {
            todo!()
        }
        
        if cmd.is_empty() || cmd.len() > 4 {
            return Err(DisplayError::BusError(Error::InvalidParameter));
        }
        
        let cmd_word = cmd.iter()
            .fold(0u32, |acc, &byte| (acc << 8) | (byte as u32));
        
        self.send_cmd(cmd_word, cmd.len() as u8, true).map_err(DisplayError::BusError)?;

        self.send_pixel_data_rect(metadata.width, metadata.height, buffer).await.map_err(DisplayError::BusError)
    }
}
