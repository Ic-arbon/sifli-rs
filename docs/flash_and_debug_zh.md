# SiFli ❤️ Rust - 烧录和调试指南

[English](flash_and_debug.md) | 中文

## 烧录指南

通常，需要烧录三个固件文件。以带有MPI2挂载的外部Flash的sf52为例，这些文件是flash table@0x12000000、bootloader@0x12010000和用户固件@0x12020000。

芯片出厂时自带引导加载程序(bootloader)，或者在烧录一次后您可以选择不再覆盖它。flash table包含有关实际固件大小的信息，所以您要么需要每次都更新它，要么在flash table中将固件大小设置得非常大。

您可以使用[sifli-flash-table](../sifli-flash-table/README.md)生成一个新的`ftab.bin`。这是Linux和Mac用户的唯一方法；对于Windows用户，您也可以使用从SDK编译的`ftab.bin`。确保新的固件大小小于它。

你可以在[这里](examples\sf32lb52x)找到准备好的ftab.bin， 它由[sifli-flash-table](../sifli-flash-table/README.md)生成。

### SFTool (适用于Linux、Mac和Windows)

Linux/Mac:

```bash
sftool -c SF32LB52 -p /dev/ttyUSB0 write_flash bootloader.bin@0x12010000 app.bin@0x12020000 ftab.bin@0x12000000
```

Windows:

```bash
sftool -c SF32LB52 -p COM7 write_flash write_flash bootloader.bin@0x12010000 app.bin@0x12020000 ftab.bin@0x12000000
```

如果文件格式包含地址信息，例如`hex`或`elf`，则@ADDRESS是可选的。

在烧录完`ftab.bin`和`bootloader.bin`后，您可以选择只烧录application（但是需要注意，您的application的bin的大小需要小于`ftab.bin`中记录的大小）。在`examples/sf32lb52x/.cargo/config.toml`中，您需要将`runner`设置为 sftool:

```bash
runner = 'sftool -c SF32LB52 -p <Your_Port> --compat write_flash'
```

然后，您就可以正常使用`cargo run`命令来烧录了：

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

详细信息请参阅[OpenSiFli/sftool](https://github.com/OpenSiFli/sftool)。

### SiFliUartDownload (仅限Windows)

首先，安装[cargo-binutils](https://github.com/rust-embedded/cargo-binutils)：

```bash
cargo install cargo-binutils
rustup component add llvm-tools
```

接下来，使用`objcopy`生成一个`.bin`文件：

```bash
cargo objcopy --bin blinky -- -O binary main.bin
```

然后，在SDK中编译[blink/no-os](https://github.com/OpenSiFli/SiFli-SDK/tree/main/example/get-started/blink/no-os)项目，并将`main.bin`文件复制到构建目录（例如`build_em-lb525_hcpu`），替换现有的`main.bin`文件。

确保新固件大小小于旧固件；否则，您可能需要手动修改`ftab`或使用[sifli-flash-table](../sifli-flash-table/README.md)生成新的`ftab.bin`。

之后，使用与SDK相同的编程方法（例如，运行`build_em-lb525_hcpu\uart_download.bat`或通过JLink进行编程）。

## 调试指南

### probe-rs (适用于Linux、Mac和Windows)

在[probe-rs](https://github.com/probe-rs/probe-rs)的最新版本(v0.28.0)中，SiFliUart调试接口已经被合并。您需要安装最新版本的probe-rs：

```bash
cargo install probe-rs-tools --force
```

要使`probe-rs`将您的串口识别为`sf32`的调试端口，请按照以下方法之一操作：

方法1：修改CH343的`production string`，使其包含关键字`SiFli`（不区分大小写）。

方法2：设置环境变量`SIFLI_UART_DEBUG=1`，然后重启软件或计算机使更改生效。使用此方法，probe-rs将识别所有串口为SiFliUart调试接口。

**目前，probe-rs无法烧录程序（不能使用`run`或`download`），只能使用`attach`。**

```bash
SIFLI_UART_DEBUG=1 probe-rs attach --chip SF32LB52 target\thumbv8m.main-none-eabi\debug\blinky
```

然后您可以看到defmt rtt日志输出并使用调试功能。

以下是一个VSCode的launch.json配置文件，供参考：

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

### SifliUsartServer (仅限Windows)

通过使用[SifliUsartServer](https://github.com/OpenSiFli/SiFli-SDK/tree/main/tools/SifliUsartServer)，您可以生成一个J-Link服务器，然后允许您使用VS Code中的Cortex-Debug连接到它。

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

**我尝试使用Jlink RTT并能够扫描defmt rtt通道，但看不到任何日志。两者之间可能存在格式差异。**

在某些HardFault场景中，Cortex-Debug连接可能会中断。如果发生这种情况，您可能需要使用J-Link Commander或其他工具进行调试。

### 注意

如果您的调试过程不稳定，这可能是由于Embassy在执行空闲任务期间使用WFI导致的。

您可以尝试在`example/sf32lb52x/Cargo.toml`中将`embassy-executor`crate的`arch-cortex-m`特性修改为`arch-spin`，将`main`函数上的`#[embassy_executor::main]`属性更新为`#[embassy_executor::main(entry = "cortex_m_rt::entry")]`。这通过不指定架构来防止芯片进入WFI。