# 使用 Nix 配置开发环境

本文档说明如何在本仓库中基于 Nix 构建一致的开发环境，并扩展额外的 Rust CLI 工具（例如通过 `cargo install` 安装的程序）。

## 进入开发环境

1. 初次进入目录时运行 `direnv allow`，确保 shell 会自动加载 `nix develop`。
2. 手动启用开发环境：
   ```bash
   nix develop
   ```
   成功后可直接使用交叉编译工具链、`rust-analyzer` 等预置工具。

## 快速查看当前工具

`flake.nix` 中的 `devShells` 定义了所有随环境提供的工具。执行以下命令可在不进入 shell 的情况下检查：

```bash
nix shell .#sftool --command sftool --help
```

或在 `nix develop` 内查看 `$PATH` 中的可执行文件。

## 添加新的 Cargo CLI（推荐流程）

> **目标**：把原本需要 `cargo install my-tool` 的程序纳入 flake，避免污染宿主环境。

优先方案（上游已有 flake）：

- 在 `flake.nix` 的 `inputs` 中加入：
  ```nix
  mytool = { url = "github:org/mytool"; inputs.nixpkgs.follows = "nixpkgs"; };
  ```
- 在 `devShells` 中按系统选择：
  ```nix
  let system = pkgs.stdenv.hostPlatform.system;
  mytoolPkg = inputs.mytool.packages.${system}.default;
  in pkgs.mkShellNoCC { packages = [ /* … */ ] ++ [ mytoolPkg ]; }
  ```
- 如需通过 `nix shell .#mytool` 直接获取，可在 `packages` 暴露同名条目：
  ```nix
  packages.${system}.mytool = mytoolPkg;
  ```

本地打包方案（上游无 flake）：

1. 在 `nix/` 目录下为工具新建 `<tool>.nix`，使用 `rustPlatform.buildRustPackage`。示例：
   ```nix
   { lib, rustPlatform, fetchFromGitHub, pkg-config }:
   rustPlatform.buildRustPackage rec {
     pname = "my-tool";
     version = "1.2.3";

     src = fetchFromGitHub {
       owner = "someone";
       repo = "my-tool";
       rev = version;
       hash = "sha256-…"; # 可先用 lib.fakeSha256
     };

     cargoHash = lib.fakeSha256; # 交由后续步骤更新
     nativeBuildInputs = [ pkg-config ]; # 依赖按需填写
   }
   ```
2. 在 `flake.nix` 的 overlay 中暴露该包，并将其加入所有 `devShells` 的 `packages`。
3. 运行一次构建以获取真实 `cargoHash`：
   ```bash
   nix build --option allow-dirty true .#my-tool
   ```
   构建失败会提示实际哈希值，按提示替换 `cargoHash`（若 `src.hash` 设为占位，也需一并更新），然后再次构建确认成功。
4. 重新进入开发环境（或 `direnv reload`）后即可直接使用 `my-tool --help` 等命令。

## 升级工具版本

1. 修改对应 `<tool>.nix` 中的 `version`、`rev`，将 `cargoHash` 重置为 `lib.fakeSha256`（或占位字符串）。
2. 重新执行 `nix build --option allow-dirty true .#<tool>` 获取新的哈希。
3. 更新完成后建议 `nix develop` 验证可执行文件是否能正常运行。

## 复用经验：`sftool`

本仓库直接复用上游 `OpenSiFli/sftool` 的 flake 输出，不再维护本地 `nix/sftool.nix`。`sftool` 已默认加入 `devShell`，也通过 `packages` 暴露为 `.#sftool`，可直接使用：

```bash
nix shell .#sftool --command sftool --help
```

## 常见问题

- **为什么不用 `cargo install`？**  
  通过 Nix 打包可以确保团队所有成员在同一个版本（包括依赖），且不会在本机 `~/.cargo/bin` 引入额外状态。

- **需要额外的系统依赖怎么办？**  
  在 `<tool>.nix` 的 `buildInputs`/`nativeBuildInputs` 中声明即可。例如 Linux 上的串口程序通常需要 `udev`，macOS 则一般不需要。

- **多平台支持**  
  `flake.nix` 默认覆盖 `x86_64/aarch64` 的 Linux 与 macOS；如有特殊平台需求，可在 overlay 中进一步调整。

如有更多需求（如添加非 Rust 工具），可依照相同思路使用 `pkgs.callPackage` 包装其它语言的构建公式，或在 `mkShell` 中直接添加 Nixpkgs 已有的包。
