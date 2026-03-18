---
priority: P3
---

# Refactor ui_system: extract egui panel sections into helper functions

## Summary

`ui_system` in `src/main.rs` is ~960 lines — a single Bevy system that builds the entire egui side panel. It works fine but is hard to navigate and review. Split the logical sections into helper functions that each take `&mut egui::Ui` plus the relevant resources.

## Current sections (approximate)

| Section | Lines | Resources used |
|---------|-------|----------------|
| Game mode selector | ~10 | `GameMode`, `OwnedMaterials`, `RocketDimensions` |
| Camera controls | ~60 | `CameraProperties` |
| Rocket dimensions | ~80 | `RocketDimensions` |
| Flight parameters | ~50 | `RocketFlightParameters` |
| Inventory/loadout (gameplay) | ~120 | `Inventory`, `Equipped`, `OwnedMotor/Tube/Nosecone` |
| Store (gameplay) | ~180 | `PlayerBalance`, `Inventory`, `Owned*` |
| Sky/lighting | ~120 | `SkyProperties`, `SkyRenderMode`, `SunDiscSettings`, `DistanceFog`, `Bloom` |
| Wind | ~30 | `WindProperties` |
| Particles | ~40 | `ParticleProperties` |
| Save/load | ~30 | `SaveState` |
| Debug info | ~40 | various queries |

## Approach

Extract each section into a free function like:

```rust
fn ui_camera_section(ui: &mut egui::Ui, camera: &mut CameraProperties) { ... }
fn ui_rocket_section(ui: &mut egui::Ui, dims: &mut RocketDimensions) { ... }
```

The top-level `ui_system` keeps the `SidePanel::left(...).show()` wrapper and calls each helper in order. No scheduling changes needed — it's purely a readability refactor.

## Non-goals

- Don't change system scheduling or split into multiple Bevy systems (unnecessary complexity for a UI builder)
- Don't move sections to separate files yet — keep them in `main.rs` unless a section grows large enough to warrant its own module

## Origin

Previously opened as GH #11 (closed — moved to local ticket per project convention).
