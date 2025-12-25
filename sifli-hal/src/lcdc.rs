use crate::{Peripheral, interrupt, peripherals, time::Hertz};
use embassy_time::{Duration, Instant, Timer};

use crate::pac::lcdc::vals;

pub use vals::{SpiLineMode, SpiClkPol, SpiClkInit, LcdFormat, LayerFormat, TargetLcd,
    LcdIntfSel, SpiLcdFormat, SpiRdMode, SpiAccessLen, SingleAccessType, Polarity, AlphaSel};

/// SPI Configuration for the LCD interface
#[derive(Debug, Clone, Copy)]
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
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            line_mode: SpiLineMode::FourLine,
            clk_polarity: SpiClkPol::Normal,
            clk_phase: SpiClkInit::Low,
            cs_polarity: Polarity::ActiveLow,
            vsyn_polarity: Polarity::ActiveLow,
        }
    }
}

/// Main configuration for the LCDC driver
#[derive(Debug, Clone)]
pub struct Config {
    /// Output pixel format (target format sent to the display)
    pub pixel_format: LcdFormat,
    /// Input layer format (format of the framebuffer in memory)
    pub layer_format: LayerFormat,
    /// Desired output SPI clock frequency in Hz
    pub frequency: Hertz,
    /// SPI specific settings
    pub spi: SpiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pixel_format: LcdFormat::Rgb565,
            layer_format: LayerFormat::RGB565,
            frequency: Hertz::mhz(30), // 30 MHz
            spi: SpiConfig::default(),
        }
    }
}

/// LCDC Driver implementation for SF32LB52x
pub struct Lcdc<'d, T: Instance> {
    _peri: crate::PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Lcdc<'d, T> {
    /// Create a new LCDC driver instance
    pub fn new(peri: impl Peripheral<P = T> + 'd) -> Self {
        crate::into_ref!(peri);
        Self { _peri: peri }
    }

    /// Initialize the LCDC peripheral
    pub fn init(&mut self, config: &Config) {
        let regs = T::regs();

        // Soft reset the LCDC controller
        regs.command().modify(|w| w.set_reset(true));
        // Wait a short delay for reset to apply (implementation detail)
        // In a real scenario, a small spin-wait might be needed here.
        regs.command().modify(|w| w.set_reset(false));

        // Calculate clock divider
        // Formula: clk_div = (src_clk + freq - 1) / freq
        // Hardware requirement: divider >= 2
        let clk_div = if config.frequency.0 > 0 {
            T::get_freq().unwrap().0.div_ceil(config.frequency.0)
        } else {
            2
        };
        let clk_div = core::cmp::max(clk_div, 2) as u8;

        // Configure LCD Interface (LCD_CONF)
        regs.lcd_conf().modify(|w| {
            w.set_lcd_intf_sel(LcdIntfSel::Spi);
            w.set_target_lcd(TargetLcd::LcdPanel0);
            w.set_lcd_format(config.pixel_format);

            // Map pixel format to SPI specific format bits
            let spi_fmt = match config.pixel_format {
                LcdFormat::Rgb332 => SpiLcdFormat::Rgb332,
                LcdFormat::Rgb565 => SpiLcdFormat::Rgb565,
                LcdFormat::Rgb888 => SpiLcdFormat::Rgb888,
                _ => SpiLcdFormat::Rgb565,
            };
            w.set_spi_lcd_format(spi_fmt);
        });

        // Configure SPI Interface (SPI_IF_CONF)
        regs.spi_if_conf().modify(|w| {
            w.set_line(config.spi.line_mode);
            w.set_clk_div(clk_div);
            w.set_spi_cs_pol(config.spi.cs_polarity);
            w.set_spi_clk_pol(config.spi.clk_polarity);
            w.set_spi_clk_init(config.spi.clk_phase);

            // Set standard SPI behaviors
            w.set_spi_cs_auto_dis(true); // Disable CS after transaction
            w.set_spi_clk_auto_dis(true); // Disable CLK when idle
            w.set_spi_cs_no_idle(true); // Keep CS active during transaction
            w.set_dummy_cycle(0);
        });

        // Configure Tearing Effect (TE) - Disabled for now
        regs.te_conf().write(|w| w.set_enable(false));

        // Release the LCD reset signal (Active Low usually, set to 1 to release)
        regs.lcd_if_conf().modify(|w| w.set_lcd_rstb(true));
    }

    /// Helper: Wait for the Single Access (Command/Param) interface to be ready.
    /// Uses a blocking wait with timeout since command transmission is very fast.
    fn wait_single_busy(&self) -> Result<(), Error> {
        let regs = T::regs();
        let start = Instant::now();
        while regs.lcd_single().read().lcd_busy() {
            if start.elapsed() > Duration::from_millis(100) {
                return Err(Error::Timeout);
            }
        }
        Ok(())
    }

    /// Helper: Wait for the Data Path (Layer/DMA) to be ready.
    /// Uses a blocking wait for now to ensure safe state before starting new transfers.
    fn wait_status_busy(&self) -> Result<(), Error> {
        let regs = T::regs();
        let start = Instant::now();
        // Check both global status and single interface status
        while regs.status().read().lcd_busy() || regs.lcd_single().read().lcd_busy() {
            if start.elapsed() > Duration::from_millis(500) {
                return Err(Error::Timeout);
            }
        }
        Ok(())
    }

    /// Send a command to the LCD via SPI.
    ///
    /// This is blocking because SPI commands are short and there is no dedicated interrupt
    /// for single command completion.
    pub fn send_cmd(&mut self, cmd: u32, len_bytes: u8) -> Result<(), Error> {
        if len_bytes == 0 || len_bytes > 4 {
            return Err(Error::InvalidParameter);
        }

        self.wait_single_busy()?;

        let regs = T::regs();

        regs.spi_if_conf().modify(|w| {
            // Set write mode to normal
            w.set_spi_rd_mode(SpiRdMode::Normal);

            // Set the length of the transaction
            let len_val = match len_bytes {
                1 => SpiAccessLen::Bytes1,
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
    pub fn send_cmd_data(&mut self, data: u32, len_bytes: u8) -> Result<(), Error> {
        if len_bytes == 0 || len_bytes > 4 {
            return Err(Error::InvalidParameter);
        }

        self.wait_single_busy()?;

        let regs = T::regs();

        regs.spi_if_conf().modify(|w| {
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
    /// The signature is `async`, but the current implementation uses a polled wait (dead wait)
    /// for the End-Of-Frame (EOF) flag to allow for fast verification without complex interrupt handling.
    pub async fn send_pixel_data(
        &mut self,
        buffer: &[u8],
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        layer_format: LayerFormat,
    ) -> Result<(), Error> {
        let regs = T::regs();

        // Ensure previous operations are complete
        self.wait_status_busy()?;

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

        // Configure Layer 0
        regs.layer0_config().write(|w| {
            w.set_active(true);
            w.set_format(layer_format);
            w.set_alpha(255); // Fully opaque
            w.set_alpha_sel(AlphaSel::Layer);
            w.set_prefetch_en(true);
            w.set_v_mirror(false);

            // Calculate width in bytes for the pitch
            let bpp = match layer_format {
                LayerFormat::RGB565 | LayerFormat::ARGB8565 => 2,
                LayerFormat::RGB888 => 3,
                LayerFormat::ARGB8888 => 4,
                _ => 2, // Default/Fallback
            };
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

        // Set Source Address
        // Note: Buffer alignment requirements (usually 2 bytes for RGB565) are assumed to be met.
        let addr = buffer.as_ptr() as u32;
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

            if start_wait.elapsed() > Duration::from_secs(1) {
                return Err(Error::Timeout);
            }

            // Yield briefly
            Timer::after(Duration::from_micros(50)).await;
        }

        Ok(())
    }
}

/// Errors that can occur during LCD operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Timeout,
    InvalidParameter,
    HardwareError(u32),
}

// ============================================================================
// Trait Definitions
// ============================================================================

pub(crate) trait SealedInstance:
    crate::rcc::RccEnableReset + crate::rcc::RccGetFreq
{
    fn regs() ->  crate::pac::lcdc::Lcdc;
}

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(SpiRstbPin, Instance);

impl SealedInstance for peripherals::LCDC1 {
    fn regs() -> crate::pac::lcdc::Lcdc {
        crate::pac::LCDC1
    }
}
impl Instance for peripherals::LCDC1 {
    type Interrupt = crate::interrupt::typelevel::LCDC1;
}