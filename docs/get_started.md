# SiFli ❤️ Rust - Getting Started

This guide will help you set up your development environment and understand the basics of working with SiFli microcontrollers using Rust.

Read [Introduction to Embedded Rust](intro_to_embedded_rust.md)

## Prerequisites

This guide assumes you have basic familiarity with:
- General programming concepts
- Microcontroller development
- Basic Rust knowledge
- Command line interfaces

## Reproducible Environment with Nix

If you want a turnkey setup, we provide a Nix flake (see `contrib/nix/`) that installs the full cross-compilation toolchain and the SiFli flashing utility. See [docs/dev_env_nix.md](dev_env_nix.md) for details on enabling the shell via `direnv`/`nix develop ./contrib/nix` and on adding extra CLI tools.

## Installing the Rust Toolchain

### 1. Install Rust

First, you'll need to install Rust using rustup, the Rust toolchain installer:

- **Windows**: Download and run [rustup-init.exe](https://win.rustup.rs/)
- **macOS/Linux**: Run the following command in your terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation.

### 2. Install the ARM Cortex-M Target

SiFli microcontrollers use the ARM Cortex-M architecture. Add the appropriate target to your Rust toolchain:

```bash
rustup target add thumbv8m.main-none-eabi
rustup target add thumbv8m.main-none-eabihf
```

## Understanding the sifli-rs Components

The sifli-rs ecosystem consists of several components:

1. **sifli-pac** - Peripheral Access Crate: Low-level register definitions generated from SVD files in [sifli-pac](https://github.com/OpenSiFli/sifli-pac)
2. **sifli-hal** - Hardware Abstraction Layer: Safe, higher-level APIs that abstract the hardware details
3. **sifli-flash-table** - Tool for generating flash tables required for firmware flashing

## Embassy Framework Overview

The SiFli HAL is designed to work with the [Embassy](https://github.com/embassy-rs/embassy) framework (but also compatible with other async executor), which provides:

- An async/await runtime for embedded devices
- A cooperative multitasking scheduler
- HAL implementations for various peripherals
- Time management and synchronization primitives

Key Embassy concepts used in this project:

- **Tasks**: Concurrent units of work (similar to threads but cooperative)
- **Spawner**: Used to spawn and manage tasks
- **Timer**: Async timing utilities for delays and timeouts

See [Embassy Book](https://embassy.dev/book/) for more.

## Project Structure

Looking at the example code for SF32LB52x:

- `.cargo/config.toml`: Target-specific configurations
- `Cargo.toml`: Project dependencies
- `memory.x`: Memory layout configuration
- `build.rs`: Build script for linker configurations
- `src/bin/`: Example applications

## Your First Project: Understanding the Blinky Example

The blinky example is the "Hello World" of embedded programming. Let's break down the code:

```rust
#![no_std]  // No standard library - we're on an embedded device
#![no_main]  // No main function - we use the cortex-m-rt entry point

use defmt::*;  // Logging framework for embedded devices
use defmt_rtt as _;  // RTT (Real-Time Transfer) backend for defmt
use panic_probe as _;  // Panic handler that logs via probe
use embassy_time::Timer;  // Async timer
use embassy_executor::Spawner;  // Task spawner

use sifli_hal;  // The SiFli HAL
use sifli_hal::gpio;  // GPIO module

#[embassy_executor::main]  // Embassy entry point
async fn main(_spawner: Spawner) {
    // Initialize the HAL with default configuration
    let p = sifli_hal::init(Default::default());

    // Create a GPIO output pin for the LED (PA26 on SF32LB52-DevKit-LCD)
    let mut led = gpio::Output::new(p.PA26, gpio::Level::Low);
    
    // Log a message via defmt
    info!("Hello World!");
    // Print clock configuration
    sifli_hal::rcc::test_print_clocks();
    
    // Main application loop
    loop {
        info!("led on!");
        led.set_high();  // Turn on the LED
        Timer::after_secs(1).await;  // Wait for 1 second asynchronously

        info!("led off!");
        led.set_low();  // Turn off the LED
        Timer::after_secs(1).await;  // Wait for 1 second asynchronously
    }
}
```

### Key Concepts:

1. **`#![no_std]` and `#![no_main]`**: These attributes indicate that we're not using the standard library or the typical Rust main function, which is common in embedded development.

2. **`embassy_executor::main`**: This attribute sets up the Embassy runtime and entry point.

3. **`sifli_hal::init`**: Initializes the hardware with the given configuration.

4. **GPIO**: The `gpio::Output::new` function creates a new GPIO output pin.

5. **Async/Await**: The `Timer::after_secs(1).await` line demonstrates async programming - it yields control back to the executor while waiting.

## UART Communication Example

The USART example demonstrates serial communication:

```rust
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = sifli_hal::init(Default::default());

    let mut config = Config::default();
    config.baudrate = 1000000;  // Set baud rate to 1Mbps
    
    // Initialize UART with RX on PA18 and TX on PA19
    let mut usart = Uart::new_blocking(p.USART1, p.PA18, p.PA19, config).unwrap();

    // Send some messages
    unwrap!(usart.blocking_write(b"Hello SiFli!\n"));
    // ...
    
    // Echo back any received bytes
    loop {
        let mut buf = [0u8; 5];
        unwrap!(usart.blocking_read(&mut buf));
        unwrap!(usart.blocking_write(&buf));
    }
}
```

### Key Concepts:

1. **UART Configuration**: The `Config` struct allows you to set parameters like baud rate.

2. **Pin Assignment**: The UART is initialized with specific pins for TX and RX.

3. **Blocking I/O**: This example uses blocking read/write operations (`blocking_read`/`blocking_write`).

4. **Non-blocking alternatives**: The project also provides a buffered UART example that uses interrupts for non-blocking operation.

## Advancing to More Complex Examples

Once you're comfortable with these basic examples, you can explore the other examples:

- **raw_rtt.rs**: Basic debug output without Embassy
- **rcc_240mhz.rs**: Clock configuration for higher performance
- **usart_buffered.rs**: Interrupt-driven UART communication

## Flashing and Debugging

For detailed instructions on how to flash your firmware and debug your application, refer to the project's [Flash and Debug Guide](../docs/flash_and_debug).

## Troubleshooting

### Common Issues:

1. **"Can't connect to the microcontroller"**: Make sure your serial port is correctly configured and recognized. See the Debug Guide for details about the SIFLI_UART_DEBUG environment variable.

2. **"Unstable debugging"**: The project notes that debugging may be unstable due to WFI (Wait For Interrupt) during idle task execution. Follow the instructions in the Debug Guide to modify Embassy's configuration.

3. **"HardFault when running my application"**: This could be related to the MSPLIM register. Ensure the `set-msplim` feature is enabled.

4. **"My UART communication is unreliable at high baud rates"**: The buffered UART example notes that interrupt-driven UART without DMA may not work reliably at 1Mbps.

## Next Steps

As you become more comfortable with the SiFli HAL and Embassy:

1. Have a Try! [Flash and Debug Guide](../docs/flash_and_debug).
2. Experiment with the other peripherals mentioned in the README status section
3. Explore the Embassy documentation to learn more about async embedded programming
4. Check the GitHub repository regularly for updates and new features
5. Consider contributing to the project by implementing or improving peripheral drivers

## Resources

- [Embassy Home Page](https://embassy.dev)
- [SiFli HAL Repository](https://github.com/OpenSiFli/sifli-rs)
- [probe-rs Documentation](https://probe.rs/docs/)

- Rust Embedded Book (highly recommended, must read!):  [English](https://docs.rust-embedded.org/book/)  [中文](https://xxchang.github.io/book/)

- Google Comprehensive-Rust, Bare Metal Programming Chapter:  [English](https://google.github.io/comprehensive-rust/bare-metal.html)  [中文](https://google.github.io/comprehensive-rust/zh-CN/bare-metal.html)

- Rust Discovery Book:  [English](https://docs.rust-embedded.org/discovery/)  [中文](https://jzow.github.io/discovery/)

- The Embedonomicon:  [English](https://docs.rust-embedded.org/embedonomicon/)   [中文](https://xxchang.github.io/embedonomicon/)

- [Embassy Blogs in The Embedded Rustacean](https://blog.theembeddedrustacean.com/series/rust-embassy)

- Intro to Embassy (Video): [Youtube](https://www.youtube.com/watch?v=pDd5mXBF4tY) [BiliBili](https://www.bilibili.com/video/BV1ZBP9enE1j)

- Embassy Book:  [English](https://embassy.dev/book/)  [中文](https://decaday.github.io/embassy-docs-zh/zh/index.html)



Happy coding with SiFli and Rust!
