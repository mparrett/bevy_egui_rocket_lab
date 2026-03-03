run:
    cargo run

debug:
    RUST_BACKTRACE=full cargo run

fmt:
    rustfmt ./src/*.rs

release:
    cargo build --release

release-wasm:
    cargo build --profile wasm-release --target wasm32-unknown-unknown
    wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/wasm-release/bevy-rocket-lab.wasm

dev-wasm:
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/debug/bevy-rocket-lab.wasm

serve-dev-wasm: dev-wasm
    python3 -m http.server 8080

serve-wasm: release-wasm
    python3 -m http.server 8080

alias serve := serve-wasm

opt-wasm: release-wasm
    wasm-opt -Oz --all-features -o out/bevy-rocket-lab_bg.wasm out/bevy-rocket-lab_bg.wasm

serve-opt-wasm: opt-wasm
    python3 -m http.server 8080

check:
    cargo check

clippy:
    cargo clippy

deps:
    cargo tree

process-assets:
    cargo run --features bevy/asset_processor
