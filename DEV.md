# Development Notes

## Module Overview

| Module | Role |
|--------|------|
| `main.rs` | App setup, plugin registration, egui UI panel, game events, input handling, state management |
| `rocket.rs` | Rocket entity spawning, body/fin/cone assembly, `RocketDimensions`, `RocketFlightParameters`, `RocketState` |
| `physics.rs` | Timed force/torque application, axis locking, physics timestep config |
| `camera.rs` | Camera follow modes (FreeLook, FixedGround, FollowSide, FollowAbove), zoom levels, orbital controls |
| `particles.rs` | Rocket exhaust/smoke effects via `bevy_firework` |
| `ground.rs` | Ground plane with tiled grass texture and collision |
| `sky.rs` | Skybox cubemap, fog, directional light, shadow cascades |
| `fin.rs` | Procedural triangular fin mesh generation |
| `cone.rs` | Procedural nose cone mesh generation |
| `rendering.rs` | UV manipulation for texture tiling |
| `fps.rs` | FPS counter overlay |
| `util.rs` | Random vector generation |
| `terrain.rs` | Procedural terrain (incomplete/commented out) |
| `_notes.rs` | Archived code snippets |

## Key Dependencies

- **avian3d** — 3D physics (forces, collisions, axis locking)
- **bevy_egui** — Immediate-mode UI panels
- **bevy_firework** — Particle system
- **bevy-inspector-egui** — World inspector debug tool (toggle with Escape)

## Assets

`assets/` contains audio (`.ogg`), fonts, textures (`.png`, `.ktx2`), environment maps, and a `.glb` mesh. Texture conversion scripts live in `scripts/` (requires [KTX-Software](https://github.com/KhronosGroup/KTX-Software/releases)).

## Toolchain

This project requires **rustup**-managed Rust (not Homebrew). Homebrew's `rust` formula doesn't support cross-compilation targets like `wasm32-unknown-unknown`.

If you have both installed, remove Homebrew's: `brew uninstall rust`

Verify with: `rustup show` — should show `stable-aarch64-apple-darwin` (or your platform) as active.

## Build Profiles

- Native `release` profile is performance-oriented: `opt-level=3`, `lto="thin"`.
- WASM `wasm-release` profile is size-oriented: `opt-level='z'`, full LTO, `panic="abort"`.

Use:

```
just release          # native performance build
just release-wasm     # wasm32 build using the wasm-release profile
```

## WASM Build

### Prerequisites

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

The `wasm-bindgen-cli` version must match the `wasm-bindgen` crate version in `Cargo.lock`. If you get a schema version mismatch, update the CLI:

```
cargo install -f wasm-bindgen-cli --version <version from error message>
```

### Build & Serve

```
just release-wasm     # build only — outputs to out/
just serve-wasm       # build + serve at http://localhost:8080
```

The WASM output (~100MB unoptimized) goes to `out/` (gitignored). `index.html` in the project root loads it.

### WASM Notes

- All deps (avian3d, bevy_egui, bevy_firework, bevy-inspector-egui) support WASM
- `getrandom` requires the `js` feature for WASM — already set in `Cargo.toml`
- Audio may require a user gesture (click) before playing in the browser
- Physics runs single-threaded on WASM — expect lower performance than native

## Troubleshooting

`cargo clean` reclaims significant disk space (~25GB with full dep cache).
