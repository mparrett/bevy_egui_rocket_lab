# Feature: WASM Build for Browser Testing

## Status: DONE (2026-02-28)

Build pipeline works. Basic browser testing confirmed.

## What was done

1. Removed Homebrew Rust (conflicts with rustup cross-compilation targets)
2. Updated rustup stable toolchain to 1.93.1
3. Added `getrandom = { version = "0.2", features = ["js"] }` to Cargo.toml
4. Updated `wasm-bindgen-cli` to 0.2.114 (must match crate version)
5. Added `just serve-wasm` target (build + serve)
6. Documented toolchain and WASM build in DEV.md
7. `index.html` already existed and works as-is

## Remaining polish

- [ ] Optimize WASM size with `wasm-opt` (currently ~100MB)
- [ ] Audio autoplay — may need user gesture before first sound plays
- [ ] Performance profiling — physics is single-threaded on WASM
- [ ] Test KTX2 compressed cubemaps (currently using PNG, which is safe)
