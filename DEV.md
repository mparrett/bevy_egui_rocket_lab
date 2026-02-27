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

- **bevy_xpbd_3d** — 3D physics (forces, collisions, axis locking)
- **bevy_egui** — Immediate-mode UI panels
- **bevy_firework** — Particle system
- **bevy-inspector-egui** — World inspector debug tool (toggle with Escape)
- **bevy_generative** — Terrain generation plugin

## Assets

`assets/` contains audio (`.ogg`), fonts, textures (`.png`, `.ktx2`), environment maps, and a `.glb` mesh. Texture conversion scripts live in `scripts/` (requires [KTX-Software](https://github.com/KhronosGroup/KTX-Software/releases)).

## WASM Build Prerequisites

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

## Troubleshooting

`cargo clean` reclaims significant disk space (~25GB with full dep cache).
