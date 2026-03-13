run:
    cargo run

watch:
    cargo watch -s 'just run'

debug:
    RUST_BACKTRACE=full cargo run

fmt:
    rustfmt ./src/*.rs

release:
    cargo build --release

# WebGPU WASM builds
release-wasm-webgpu:
    cargo build --profile wasm-release --target wasm32-unknown-unknown --features web_webgpu
    wasm-bindgen --out-dir ./out-webgpu/ --target web ./target/wasm32-unknown-unknown/wasm-release/bevy-rocket-lab.wasm

dev-wasm-webgpu:
    cargo build --target wasm32-unknown-unknown --features web_webgpu
    wasm-bindgen --out-dir ./out-webgpu/ --target web ./target/wasm32-unknown-unknown/debug/bevy-rocket-lab.wasm

# WebGL2 WASM builds
release-wasm-webgl:
    cargo build --profile wasm-release --target wasm32-unknown-unknown --features web_webgl
    wasm-bindgen --out-dir ./out-webgl/ --target web ./target/wasm32-unknown-unknown/wasm-release/bevy-rocket-lab.wasm

dev-wasm-webgl:
    cargo build --target wasm32-unknown-unknown --features web_webgl
    wasm-bindgen --out-dir ./out-webgl/ --target web ./target/wasm32-unknown-unknown/debug/bevy-rocket-lab.wasm

# Backwards-compat aliases (default to webgpu)
release-wasm: release-wasm-webgpu

dev-wasm: dev-wasm-webgpu

# Both builds
release-wasm-all: release-wasm-webgpu release-wasm-webgl

dev-wasm-all: dev-wasm-webgpu dev-wasm-webgl

# Serve targets
serve-dev-wasm: dev-wasm-all
    python3 -m http.server 8080

serve-dev-wasm-webgl: dev-wasm-webgl
    python3 -m http.server 8080

serve-wasm: release-wasm-all
    python3 -m http.server 8080

alias serve := serve-wasm

opt-wasm: release-wasm-all
    wasm-opt -Oz --all-features -o out-webgpu/bevy-rocket-lab_bg.wasm out-webgpu/bevy-rocket-lab_bg.wasm
    wasm-opt -Oz --all-features -o out-webgl/bevy-rocket-lab_bg.wasm out-webgl/bevy-rocket-lab_bg.wasm

serve-opt-wasm: opt-wasm
    python3 -m http.server 8080

check:
    cargo check

test:
    cargo test

clippy:
    cargo clippy

deps:
    cargo tree

process-assets:
    cargo run --features bevy/asset_processor
