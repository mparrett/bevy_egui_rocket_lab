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
just check            # cargo check
just test             # cargo test (ECS regression tests in src/main.rs)
just clippy           # cargo clippy
just dev-wasm         # fast WASM build (dev profile, no LTO)
just serve-dev-wasm   # dev-wasm + serve on :8080
just release-wasm     # wasm32 + wasm-bindgen using the wasm-release profile
just serve-wasm       # release-wasm + serve on :8080
just serve-opt-wasm   # release-wasm + wasm-opt -Oz + serve on :8080
just deps             # cargo tree
```

Tests exist (`just test`) for launch/reset/landing core-loop behavior. Use `just serve-dev-wasm` + the webapp-testing skill to verify WASM builds in a browser before pushing. Use `just serve-wasm` (wasm-release profile) for final size-optimized web builds.

## Workflow

**Always test locally before pushing.** This project auto-deploys to GitHub Pages on push to main. Run `just run` for native or `just serve-wasm` for WASM, and verify behavior before committing/pushing. Don't batch untested changes into a push.

**Before every push**, run `just clippy` and `just test` — these same checks run in CI (`.github/workflows/ci.yml`) and will block PRs if they fail.

**Running the app:** Always run via `just run` (or `just debug`, `just serve-wasm`, etc.) inside the tmux session named `rocket-lab-runner`.

## Architecture

**State & Messages:** `LaunchEvent`, `DownedEvent`, `ResetEvent` messages for rocket lifecycle. `RocketStateEnum` tracks `Initial → Launched → Grounded`.

**Resources as config:** `RocketDimensions`, `RocketFlightParameters`, `CameraProperties`, `SkyProperties` are Bevy resources modified via the egui panel and read by systems.

**System scheduling:** Startup systems spawn entities (ground, camera, rocket, sky). Update systems handle input, forces, UI, and position tracking. PostUpdate handles camera transforms. See ADR-005 in `decisions.md` for physics system scheduling rules (force systems in `PhysicsSystems::First`, constraints after `StepSimulation`, visuals after `Writeback`).

**Rocket entity tree:** Rocket is a parent entity with child entities for body cylinder, cone, and fins. Fins are rebuilt dynamically when dimensions change. Dynamic bodies tracked by visuals need `TransformInterpolation`.

## Clippy

`type_complexity` and `too_many_arguments` are allowed project-wide in `Cargo.toml`.

## Project Memory

Memory files live in `docs/project_notes/`.

**Before proposing changes**: Check `decisions.md` for existing ADRs
**When encountering errors**: Search `bugs.md` for known solutions
**When looking up config**: Check `key_facts.md` for ports, URLs, environments

When resolving bugs or making decisions, update the relevant file.
