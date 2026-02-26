# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bevy Rocket Lab is a model rocketry simulator/game built with Bevy 0.13.1 (Rust). It features physics-based rocket flight, procedural mesh generation, particle effects, and an egui control panel. Early stage — currently a sandbox/tech demo.

## Build Commands

All tasks are defined in `Justfile`:

```bash
just run              # cargo run (dev build with opt-level=1, deps at opt-level=3)
just debug            # cargo run with RUST_BACKTRACE=full
just fmt              # rustfmt ./src/*.rs
just release          # cargo build --release
just release-wasm     # build for wasm32-unknown-unknown + wasm-bindgen
just server           # python3 http server on :8080 (for testing WASM builds)
just deps             # cargo tree
just process-assets   # cargo run --features bevy/asset_processor
```

WASM prerequisites: `rustup target add wasm32-unknown-unknown` and `cargo install wasm-bindgen-cli`.

No test suite exists yet.

## Architecture

Single-crate Bevy app. All source is in `src/` with modules registered in `main.rs`.

### Module Responsibilities

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

### Key Patterns

**State & Events:** `GameState` enum (currently just `Initial`), plus `LaunchEvent`, `DownedEvent`, `ResetEvent` for rocket lifecycle. `RocketStateEnum` tracks `Initial → Launched → Grounded`.

**Resources as config:** `RocketDimensions`, `RocketFlightParameters`, `CameraProperties`, `SkyProperties` are Bevy resources modified via the egui panel and read by systems.

**System scheduling:** Startup systems spawn entities (ground, camera, rocket, sky). Update systems handle input, forces, UI, and position tracking. PostUpdate handles camera transforms.

**Rocket entity tree:** Rocket is a parent entity with child entities for body cylinder, cone, and fins. Fins are rebuilt dynamically when dimensions change.

### Key Dependencies

- **bevy_xpbd_3d** — 3D physics (forces, collisions, axis locking)
- **bevy_egui** — Immediate-mode UI panels (pinned to 0.25 due to [#269](https://github.com/mvlabat/bevy_egui/issues/269))
- **bevy_firework** — Particle system (pinned to git rev for unreleased feature from [PR #12](https://github.com/mbrea-c/bevy_firework/pull/12))
- **bevy-inspector-egui** — World inspector debug tool (toggle with Escape)
- **bevy_generative** — Terrain generation plugin

## Clippy Lints

`type_complexity` and `too_many_arguments` are allowed project-wide in `Cargo.toml`.

## Assets

`assets/` contains audio (`.ogg`), fonts, textures (`.png`, `.ktx2`), environment maps, and a `.glb` mesh. Texture conversion scripts live in `scripts/`.
