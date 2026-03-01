run:
    cargo run

debug:
    RUST_BACKTRACE=full cargo run

fmt:
    rustfmt ./src/*.rs

release:
    cargo build --release

release-wasm:
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/bevy-rocket-lab.wasm

serve-wasm: release-wasm
    python3 -m http.server 8080

server:
    python3 -m http.server 8080

deps:
    cargo tree

process-assets:
    cargo run --features bevy/asset_processor
