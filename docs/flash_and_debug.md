# SiFli ❤️ Rust - Flash and Debug Guide

[中文](flash_and_debug_zh.md) | English

## Flash Guide

Typically, three firmware files need to be flashed. Taking sf52 with MPI2-mounted external Flash as an example, these are flash table@0x12000000, bootloader@0x12010000, and user firmware@0x12020000.

The chip comes with a bootloader from the factory, or you can choose not to overwrite it after flashing it once. The flash table contains information about the actual firmware size, so you either need to update it each time or set the firmware size in the flash table to be very large.

You can use [sifli-flash-table](../sifli-flash-table/README.md) to generate a new `ftab.bin`. (This is the only method for Linux and Mac users; for Windows, you can also use the `ftab.bin` compiled from the SDK. Make sure the new firmware size is smaller than it)

### SFTool (Available on Linux, Mac and Windows)

Linux/Mac:

```bash
sftool -c SF32LB52 -p /dev/ttyUSB0 write_flash bootloader.bin@0x12010000 app.bin@0x12020000 ftab.bin@0x12000000
```

Windows:

```bash
sftool -c SF32LB52 -p COM7 write_flash write_flash bootloader.bin@0x12010000 app.bin@0x12020000 ftab.bin@0x12000000
```

@ADDRESS is optional if the file format contains address information, for example, `hex` or `elf`.

See [OpenSiFli/sftool](https://github.com/OpenSiFli/sftool) for details.

After flashing `ftab.bin` and `bootloader.bin`, you can choose to flash only the application (but be aware that the size of your application's bin must be smaller than the size recorded in `ftab.bin`). In `examples/sf32lb52x/.cargo/config.toml`, you need to set the `runner` to sftool:

```bash
runner = 'sftool -c SF32LB52 -p <Your_Port> --compat write_flash'
```

Then, you can use the `cargo run` command to flash as usual:

```shell
$ cargo run --bin blinky

    ...
    ...
    Finished `release` profile [optimized] target(s) in 0.16s
     Running `sftool -c SF32LB52 --port /dev/cu.xxxxxx --compat write_flash target/thumbv8m.main-none-eabi/release/blinky`
[0x00]   Connected success!
[0x01]   Download stub success!
[0x02]   Need to re-download
[0x03] Download at Download success!... ===================================================================================================== 15.16 KiB/s 100.000%
[0x04]   Verify success!
```

### SiFliUartDownload (Windows Only)

First, install [cargo-binutils](https://github.com/rust-embedded/cargo-binutils):

```bash
cargo install cargo-binutils
rustup component add llvm-tools
```

Next, use `objcopy` to generate a `.bin` file:

```bash
cargo objcopy --bin blinky -- -O binary main.bin
```

Then, compile the [blink/no-os](https://github.com/OpenSiFli/SiFli-SDK/tree/main/example/get-started/blink/no-os) project in the SDK and copy the `main.bin` file into the build directory (e.g., `build_em-lb525_hcpu`), replacing the existing `main.bin` file.

Make sure the new firmware size is smaller than the old one; otherwise, you may need to manually modify the `ftab` or use [sifli-flash-table](../sifli-flash-table/README.md) to generate a new `ftab.bin`.

Afterward, use the same programming method as with the SDK (for example, running `build_em-lb525_hcpu\uart_download.bat` or programming via JLink).

## Debug Guide

### probe-rs (Available on Linux, Mac and Windows)

In the latest version of [probe-rs](https://github.com/probe-rs/probe-rs) (v0.28.0), the SiFliUart debug interface has been merged. You need to install the latest version of probe-rs:

```bash
cargo install probe-rs-tools --force
```

To have `probe-rs` recognize your serial port as the debug port for `sf32`, follow one of these methods:

Method 1: Modify the `production string` of your CH343 to include the keyword `SiFli` (case-insensitive).

Method 2: Set the environment variable `SIFLI_UART_DEBUG=1`, then restart the software or your computer for the changes to take effect. With this method, probe-rs will recognize all serial ports as SiFliUart debug interfaces.

**Currently, probe-rs cannot flash programs (can't use `run` or `download`), only `attach` can be used.**

```bash
SIFLI_UART_DEBUG=1 probe-rs attach --chip SF32LB52 target\thumbv8m.main-none-eabi\debug\blinky
```

Then you can see defmt rtt log output and use debugging.

Here is a reference VSCode launch.json configuration file:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "probe-rs-debug",
            "request": "attach",
            "name": "probe_rs attach sf32lb52",
            "chip": "SF32LB52",
            "probe": "1a86:55d3:<Your_Port>",
            "coreConfigs": [
                {
                    "coreIndex": 0,
                    "programBinary": "examples/sf32lb52x/target/thumbv8m.main-none-eabi/debug/blinky",
                    "rttEnabled": true,
                    "rttChannelFormats": [
                      {
                        "channelNumber": 0,
                        "dataFormat": "String",
                        "showTimestamps": true
                      },
                      {
                        "channelNumber": 1,
                        "dataFormat": "BinaryLE"
                      }
                    ]
                },
            ],
            "env": {
                "RUST_LOG": "info",
                "SIFLI_UART_DEBUG": "1",
            },
        }
    ]
}
```

### SifliUsartServer (Windows Only)

By utilizing [SifliUsartServer](https://github.com/OpenSiFli/SiFli-SDK/tree/main/tools/SifliUsartServer), you can generate a J-Link server, which then allows you to connect to it using Cortex-Debug within VS Code.

```json
"configurations": [
        {
            "cwd": "${workspaceFolder}",
            "name": "Cortex Debug",
            "request": "attach",
            "type": "cortex-debug",
            "device": "Cortex-M33",
            "runToEntryPoint": "entry",
            "showDevDebugOutput": "none",
            "servertype": "jlink",
            "serverpath": "xxx/Dev/Jlink/JLink_V812e/JLinkGDBServerCL.exe",
            "ipAddress": "127.0.0.1:19025",
            "interface": "swd",
            "svdFile": "xxx/sifli-pac/svd/SF32LB52x.svd",
            "executable": "examples/sf32lb52x/target/thumbv8m.main-none-eabi/debug/blinky"
        },
    ]
```

**I tried using Jlink RTT and was able to scan the defmt rtt channel, but couldn't see any logs. There might be format differences between the two.**

In certain HardFault scenarios, the Cortex-Debug connection may be interrupted. If this occurs, you might need to resort to J-Link Commander or alternative tools for debugging.

### Note

If your debugging process is unstable, this may be due to Embassy  using WFI during idle task execution.  

You can try modifying the `embassy-executor` crate's `arch-cortex-m`  feature in `example/sf32lb52x/Cargo.toml` to `arch-spin`, update the `#[embassy_executor::main]` attribute on the `main` function to `#[embassy_executor::main(entry = "cortex_m_rt::entry")]`.  This prevents the chip from entering WFI by not specifying an architecture.  

