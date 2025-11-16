Convert C arrays to Rust const arrays

Usage
- LCPU patch helper (preserve original SiFli patch array names): `carray2rs --alias-patch lcpu_patch.c > patch_data.rs`

Notes
- For patch arrays like `g_lcpu_patch_list` / `g_lcpu_patch_bin`, the tool keeps the original semantic names and converts them to Rust constants such as `G_LCPU_PATCH_LIST_U32` / `G_LCPU_PATCH_BIN_U32`, so HAL-side parameters can follow the same naming.
- Supports common C types: `uint8_t`/`unsigned char` -> `u8`, `uint32_t`/`unsigned int` -> `u32`.
- Strips C/C++ comments and whitespace.
- Parses hex (`0x...`) and decimal numbers with optional `U/L` suffixes.
- Unknown types default to `u32`.
