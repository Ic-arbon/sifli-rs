# Introduction to Embedded Rust

## Overview

Rust is a systems programming language that focuses on safety, speed, and concurrency - making it an excellent choice for embedded systems development. For C developers working in the embedded space, Rust offers familiar low-level control with additional compile-time guarantees that help prevent common bugs like buffer overflows, null pointer dereferences, and data races.

The Rust embedded ecosystem has been growing steadily, with strong support for many popular microcontroller families including ARM Cortex-M, RISC-V, and others, and now including SiFli MCUs.

## Key Features and Benefits

### Memory Safety without Runtime Overhead

Rust's ownership system ensures memory safety without requiring a garbage collector. The compiler performs rigorous checks at compile time, preventing many common bugs:

- No dangling pointers or null pointer dereferences
- No buffer overflows
- No data races in concurrent code
- No use-after-free vulnerabilities

These safety features come with zero runtime cost, making Rust suitable for resource-constrained embedded systems.

### Zero-Cost Abstractions

Rust allows high-level abstractions that compile down to efficient machine code:

```rust
// Higher-level code with iterators
let sum: u32 = my_array.iter().filter(|&x| x % 2 == 0).sum();
```

Compiles to roughly the same efficiency as hand-written C:

```c
uint32_t sum = 0;
for (size_t i = 0; i < array_len; i++) {
    if (my_array[i] % 2 == 0) sum += my_array[i];
}
```

### Type System and Error Handling

Rust's type system helps catch more bugs at compile time:

- Enums with associated data replace error-prone C unions
- Pattern matching ensures exhaustive handling of all possible cases
- The `Result` type forces explicit error handling, preventing ignored error conditions, The `Option` type eliminates null pointer bugs

### Modern Development Experience

Embedded Rust provides a modern development experience with:

- A powerful package manager (Cargo) for dependency management
- Comprehensive documentation via `rustdoc` and docs.rs
- Strong compiler error messages that guide you to fixes
- A growing ecosystem of tools and libraries

### Embedded Rust vs Traditional Embedded Development

| Aspect                 | Embedded Rust                       | Traditional (C)                   |
| ---------------------- | ----------------------------------- | --------------------------------- |
| Development Experience | Modern tooling, cargo               | Varies widely by platform         |
| Abstract ability       | Relatively high                     | Relatively low                    |
| Code Reuse             | Strong module system, crates        | Often project-specific            |
| Community/Ecosystem    | Relatively weak but growing rapidly | Mature but fragmented             |
| Learning Curve         | Steeper initial curve               | Well-established patterns         |
| Type Safety            | Strong, expressive type system      | Limited, often requires macros    |
| Memory Safety          | Compile-time guarantees             | Manual checking, tools like MISRA |
| Abstraction Cost       | Zero-cost abstractions              | Often requires runtime overhead   |
| Concurrency            | Guaranteed safe at compile time     | Manual synchronization            |

## Working with Hardware

### Register Access

Instead of using preprocessor-based register definitions common in C, Rust uses type-safe peripheral access crates (PACs) generated from SVD files:

```c
uint32_t cr1_value = *cr1_reg;
uint32_t cr3_value = *cr3_reg;
uint32_t isr_value = *isr_reg;

cr1_value &= ~USART_CR1_UE;          // Disable UART during config
cr1_value |= USART_CR1_TE;           // Enable transmitter
cr1_value &= ~USART_CR1_M;
if (cr3_value & USART_CR3_HDSEL) {   // Check if half-duplex is enabled in CR3
    cr1_value |= USART_CR1_RE;       // Enable receiver
} else {
    cr1_value &= ~USART_CR1_RE;      // Disable receiver
}
cr1_value |= USART_CR1_RXNEIE;       // Enable RX not empty interrupt
*cr1_reg = cr1_value;                // Write back to CR1

cr3_value |= USART_CR3_HDSEL;        // Enable half-duplex mode
if (!(isr_value & USART_ISR_ORE)) {  // Check if overrun error is clear
    cr3_value |= USART_CR3_EIE;      // Enable error interrupt
} else {
    cr3_value &= ~USART_CR3_EIE;     // Disable error interrupt
}
*cr3_reg = cr3_value;                // Write back to CR3
```

Rust version:

```rust
// Configure USART CR1 and CR3 for half-duplex mode with error handling
r.cr1().modify(|w| {
    w.set_ue(false);           // Disable UART during config
    w.set_m(vals::M::Bit8);
    w.set_re(r.cr3().read().hdsel()); // Receiver enabled only if half-duplex is set
});
r.cr3().modify(|w| {
    w.set_hdsel(true);         // Enable half-duplex mode
    w.set_eie(!r.isr().read().ore()); // Enable error interrupt if no overrun
});
```

The Rust approach:

- Uses type-safe interfaces
- Prevents read-modify-write race conditions
- Documents register functionality through type interfaces
- Prevents accessing invalid registers or using incorrect bit masks

### Hardware Abstraction Layers (HALs)

HALs build on top of PACs to provide more ergonomic APIs:

```rust
// Using a HAL to toggle an LED
let p = sifli_hal::init(Default::default());
let mut led = gpio::Output::new(p.PA26, gpio::Level::Low);

// Turn LED on/off
led.set_high();
// ...later...
led.set_low();
```

HALs typically provide:

- Pin configuration with type-level guarantees (preventing invalid states)
- Abstracted peripheral interfaces (UART, SPI, I2C, etc.)
- Timer abstractions
- Power management utilities

### Interrupt Handling

Rust provides a safer approach to interrupt handling:

```rust
// Define interrupt handler for USART1
#[interrupt]
fn USART1() {
    // Safe access to static resources via critical sections
    cortex_m::interrupt::free(|cs| {
        if let Some(buf) = G_BUFFER.borrow(cs).as_mut() {
            // Handle USART interrupt...
        }
    });
}
```

Rust embedded frameworks often provide additional abstractions for interrupt handling:

```rust
// Embassy framework approach with interrupts bound to a peripheral
bind_interrupts!(struct Irqs {
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
});
```

## Asynchronous Programming

Rust offers robust async/await syntax for concurrent programming without an RTOS:

```rust
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize hardware
    let p = sifli_hal::init(Default::default());
    let mut led = gpio::Output::new(p.PA26, gpio::Level::Low);
    
    // Concurrent tasks
    loop {
        led.set_high();
        Timer::after_secs(1).await;  // Non-blocking delay

        led.set_low();
        Timer::after_secs(1).await;
    }
}
```

The Embassy framework provides:

- Task scheduling without a traditional RTOS
- Efficient cooperative multitasking
- Async drivers for peripherals
- Time handling utilities

## Embedded Rust Ecosystem

In general, the Embedded Rust ecosystem is relatively weak, but the community is active and growing rapidly.

### Key Tools and Libraries

- **probe-rs**: Debugging and flashing utility supporting many probe types
- **defmt**: Efficient logging framework for embedded systems
- **embedded-hal**: Traits defining standard embedded interfaces
- **cortex-m**: Low-level access to ARM Cortex-M processors
- **cortex-m-rt**: Startup code and runtime for Cortex-M devices

### Frameworks and Libraries

- **Embassy**: Asynchronous embedded framework with HALs for various chips
- **RTIC**: Real-Time Interrupt-driven Concurrency framework

## Resources

- Rust Embedded Book (highly recommended):  [English](https://docs.rust-embedded.org/book/)  [中文](https://xxchang.github.io/book/)

- Google Comprehensive-Rust, Bare Metal Programming Chapter:  [English](https://google.github.io/comprehensive-rust/bare-metal.html)  [中文](https://google.github.io/comprehensive-rust/zh-CN/bare-metal.html)

- Rust Discovery Book:  [English](https://docs.rust-embedded.org/discovery/)  [中文](https://jzow.github.io/discovery/)

- The Embedonomicon:  [English](https://docs.rust-embedded.org/embedonomicon/)   [中文](https://xxchang.github.io/embedonomicon/)

- [Embassy Blogs in The Embedded Rustacean](https://blog.theembeddedrustacean.com/series/rust-embassy)

- Intro to Embassy (Video): [Youtube](https://www.youtube.com/watch?v=pDd5mXBF4tY) [BiliBili](https://www.bilibili.com/video/BV1ZBP9enE1j)

- Embassy Book:  [English](https://embassy.dev/book/)  [中文](https://decaday.github.io/embassy-docs-zh/zh/index.html)

## Conclusion

For C embedded developers, Rust offers a path to more reliable embedded software while maintaining the performance and low-level control needed for embedded systems. The strong type system, memory safety guarantees, and modern tooling provide substantial benefits, while the growing ecosystem of HALs, PACs, and frameworks makes embedded Rust increasingly accessible.

The SiFli HAL represents one example of Rust's expanding support for diverse microcontroller families, offering embedded C developers a practical way to explore Rust's benefits in their projects.

## Let's [Get Started](get_started.md)!
