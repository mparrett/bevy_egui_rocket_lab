# Environment Setup

...

# Notes / Troubleshooting

1. Wasm32 target

```
  = note: the `wasm32-unknown-unknown` target may not be installed
```

```
rustup target add wasm32-unknown-unknown
```

2. Wasm bindgen

```
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/rocket.wasm
sh: wasm-bindgen: command not found
```

```
cargo install wasm-bindgen-cli
```

Cargo clean does a lot.

```
❯ cargo clean
     Removed 33990 files, 25.2GiB total
```
