# SiFli Flash Table

A command-line tool (Cli) used to generate a flash table firmware for SiFli MCUs.

## Usage

```bash
sifli-flash-table gen --ptab test\em-lb525\ptab.json --output ftab.bin
```

This is functionally equivalent to:

- SiFli-SDK [GenFtabCFile](https://github.com/OpenSiFli/SiFli-SDK/blob/8f42a6916c55c6b44ec45e1c0d137b15f7fa7fa3/tools/build/resource.py#L684)
- SiFli-SDK [mem_map.h](https://github.com/OpenSiFli/SiFli-SDK/blob/8f42a6916c55c6b44ec45e1c0d137b15f7fa7fa3/drivers/cmsis/sf32lb52x/mem_map.h)

The difference is that `sifli-flash-table` generates the flash table using code, instead of cross-compiling the table.

Other references:

- SiFli-SDK [boot_loader](https://github.com/OpenSiFli/SiFli-SDK/blob/8f42a6916c55c6b44ec45e1c0d137b15f7fa7fa3/example/boot_loader/project/butterflmicro/board)

- SiFli-SDK [flash table中，flash bootloader的地址问题和宏覆盖问题 · Issue #10](https://github.com/OpenSiFli/SiFli-SDK/issues/10)
- SiFli Docs [应用程序启动流程 - SiFli SDK编程指南 文档](https://docs.sifli.com/projects/sdk/v2.3/sf32lb52x/app_development/startup_flow_sf32lb52x.html)
- SiFli Docs [安全引导加载 - SiFli SDK编程指南 文档](https://docs.sifli.com/projects/sdk/v2.3/sf32lb52x/bootloader.html)

## Test

In [lib.rs](src/lib.rs), there is a test called `test_ptab_ftab_conversion`, which automatically tests the development board's flash table generation in the [test](test) folder and compares it with the precompiled `ftab.bin` from the SDK.

## TODO

- Use real user code and bootloader firmware size values
- Write automated tests for more boards
- Verify if addresses starting with `0x6xxx_xxxx` are use

## License

This project is under Apache License, Version 2.0 ([LICENSE](../LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).