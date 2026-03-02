# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bevy 0.18 (Rust) model rocketry simulator/game. Early stage sandbox/tech demo. Single crate, all source in `src/`, modules registered in `main.rs`.

## Build Commands

```bash
just run              # cargo run (dev: opt-level=1, deps: opt-level=3)
just debug            # cargo run with RUST_BACKTRACE=full
just fmt              # rustfmt ./src/*.rs
just release          # cargo build --release
just release-wasm     # wasm32-unknown-unknown + wasm-bindgen
just serve-wasm       # release-wasm + serve on :8080
just serve-opt-wasm   # release-wasm + wasm-opt -Oz + serve on :8080
just deps             # cargo tree
```

No test suite exists yet. Use the webapp-testing skill to verify WASM builds in a browser before pushing.

## Workflow

**Always test locally before pushing.** This project auto-deploys to GitHub Pages on push to main. Run `just run` for native or `just serve-wasm` for WASM, and verify behavior before committing/pushing. Don't batch untested changes into a push.

## Architecture

**State & Messages:** `LaunchEvent`, `DownedEvent`, `ResetEvent` messages for rocket lifecycle. `RocketStateEnum` tracks `Initial → Launched → Grounded`.

**Resources as config:** `RocketDimensions`, `RocketFlightParameters`, `CameraProperties`, `SkyProperties` are Bevy resources modified via the egui panel and read by systems.

**System scheduling:** Startup systems spawn entities (ground, camera, rocket, sky). Update systems handle input, forces, UI, and position tracking. PostUpdate handles camera transforms.

**Rocket entity tree:** Rocket is a parent entity with child entities for body cylinder, cone, and fins. Fins are rebuilt dynamically when dimensions change.

## Clippy

`type_complexity` and `too_many_arguments` are allowed project-wide in `Cargo.toml`.

## Project Memory

Memory files live in `docs/project_notes/`.

**Before proposing changes**: Check `decisions.md` for existing ADRs
**When encountering errors**: Search `bugs.md` for known solutions
**When looking up config**: Check `key_facts.md` for ports, URLs, environments

When resolving bugs or making decisions, update the relevant file.
