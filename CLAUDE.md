# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bevy 0.13.1 (Rust) model rocketry simulator/game. Early stage sandbox/tech demo. Single crate, all source in `src/`, modules registered in `main.rs`.

## Build Commands

```bash
just run              # cargo run (dev: opt-level=1, deps: opt-level=3)
just debug            # cargo run with RUST_BACKTRACE=full
just fmt              # rustfmt ./src/*.rs
just release          # cargo build --release
just release-wasm     # wasm32-unknown-unknown + wasm-bindgen (see DEV.md for prereqs)
just deps             # cargo tree
```

No test suite exists yet.

## Architecture

**State & Events:** `GameState` enum (currently just `Initial`), plus `LaunchEvent`, `DownedEvent`, `ResetEvent` for rocket lifecycle. `RocketStateEnum` tracks `Initial → Launched → Grounded`.

**Resources as config:** `RocketDimensions`, `RocketFlightParameters`, `CameraProperties`, `SkyProperties` are Bevy resources modified via the egui panel and read by systems.

**System scheduling:** Startup systems spawn entities (ground, camera, rocket, sky). Update systems handle input, forces, UI, and position tracking. PostUpdate handles camera transforms.

**Rocket entity tree:** Rocket is a parent entity with child entities for body cylinder, cone, and fins. Fins are rebuilt dynamically when dimensions change.

## Dependency Pins

- **bevy_egui** pinned to 0.25 — cannot upgrade due to [#269](https://github.com/mvlabat/bevy_egui/issues/269)
- **bevy_firework** pinned to git rev `a191fd8` — relies on unreleased feature from [PR #12](https://github.com/mbrea-c/bevy_firework/pull/12)

## Clippy

`type_complexity` and `too_many_arguments` are allowed project-wide in `Cargo.toml`.
