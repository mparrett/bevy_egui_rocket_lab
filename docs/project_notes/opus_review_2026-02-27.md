# Code Review — 2026-02-27

**Reviewer**: Claude Opus 4.6
**Scope**: Full codebase review (~2,750 lines across 14 source files)
**Build status**: Compiles with 4 warnings (dead code)

## Summary

Model rocketry simulator built with Bevy 0.13.1. Functional prototype: you can tweak rocket dimensions via egui, launch with timed thrust, watch particle exhaust, and track max altitude/velocity across multiple camera modes. Physics via bevy_xpbd_3d, skybox with fog, ground collision, audio.

Good modular structure for a prototype. Main issues are dead code accumulation, some architectural inconsistencies, and Bevy version drift (0.13.1 → current 0.15+).

## What Works

- Rocket spawn with live dimension tweaking (mesh + collider rebuild)
- Launch with timed thrust force synced to rocket rotation
- Crash detection with audio
- Particle effects: sparks, active smoke, residual smoke (bevy_firework)
- 4 camera follow modes with spring interpolation
- Fog cycling (3 modes)
- Slow-motion toggle
- FPS counter, scoreboard overlay
- Background music, launch/crash SFX
- World inspector (Escape toggle)
- egui side panel for all parameters

## Dead Code & Stubs

| Location | Issue |
|----------|-------|
| `main.rs:424-452` | `update_rocket_ccd_system` + `remove_rocket_ccd_system` — registered in Update, entirely commented out internally. No-op query cycles. |
| `terrain.rs` | 82 lines, never called. `TerrainPlugin` registered but `spawn_terrain` is dead. |
| `_notes.rs` | 71 lines of archived code in comments. Not compiled. |
| `rocket.rs:197-205` | `locked_axes()` fn returns unit `()` — the expression result is discarded. Actual lock logic is in `physics::lock_all_axes()`. |
| `main.rs:689-706` | Bottom egui panel shows "TODO" literal, nothing else. |
| `sky.rs:138-143` | `FocalPoint` struct — never constructed. |
| `rocket.rs:101-106` | `RocketState` fields `is_ignited`, `is_grounded`, `additional_mass`, `engine_angle` — never read. `RocketStateEnum` does the actual work. |

## Architecture Issues

### main.rs is a monolith (877 lines)

Handles UI layout, input, dimension updates, camera setup, text setup, music spawning, launch logic, crash handling, stats, and CCD stubs. The other modules are well-factored — main.rs needs the same treatment.

**Suggested splits:**
- `input.rs` — keyboard handling (`init_egui_ui_input_system`, `do_launch_system`, `adjust_time_scale`)
- `ui.rs` — egui panel layout (`ui_system`, `setup_text_system`, `update_stats_system`)
- `events.rs` or fold into `rocket.rs` — `on_launch_event`, `on_crash_event`, `rocket_position_system`

### Inconsistent input handling

Some systems read keys through `EguiContexts` (`ctx.input(|i| ...)`), others through Bevy's `ButtonInput<KeyCode>`. The egui path means keys only register when egui has focus. Sky's fog toggle uses `ButtonInput` correctly — should standardize.

### Dual force tracking mechanisms

`ForceTimer` exists as both:
1. A `Component` queried by `update_forces_system` (this does the work)
2. Items in `TimedForces.forces_set: HashSet<ForceTimer>` (only ever `.clear()`ed, never iterated for physics)

The `TimedForces` component + HashSet is vestigial. Only the `ForceTimer` component matters.

### Manual change detection

`RocketDimensions` uses `flag_changed: bool` instead of Bevy's built-in `Res::is_changed()`. Fragile — if you forget to set the flag, changes are silently ignored.

## Bugs / Correctness Issues

### Camera interpolation ignores delta_t

`camera.rs:250-259` — `interpolate_to_target_alt` takes `delta_t` but never uses it. It snaps to `target_vec * follow_lag_ratio`, making FollowSide camera behavior frame-rate dependent.

### Narrow crash detection

`main.rs:394-397`:
```rust
if velocity.y.abs() < 1.0 && velocity.y.abs() > 0.1 && y < 0.2 && state == Launched
```
A fast vertical crash (|vy| > 1.0) or a perfectly still landing (|vy| < 0.1) won't trigger detection.

### Force sync uses scalar multiply instead of vector

`physics.rs:87`:
```rust
external_force.apply_force(rotation.mul_vec3(Vec3::Y) * force.force.unwrap());
```
This multiplies a Vec3 direction by a Vec3 force (component-wise), not by a scalar magnitude. Works only because the force is `Vec3::Y * scalar` — the x/z components are zero. Fragile if force vectors change.

### Duplicate MeshData structs

`cone.rs` and `fin.rs` each define identical private `MeshData` structs. Should be shared.

## Style Issues

- ~20 `println!` calls used for debug output. Should be `info!`/`debug!` macros or removed.
- `AppExit` sent as unit struct — deprecated form in Bevy 0.13 (should be `AppExit::Success`).
- Commented-out code blocks scattered throughout (camera orbit math, extra ground plane, debug prints).

## Bevy Version Gap

Pinned to Bevy 0.13.1 (March 2024). Current Bevy is 0.15+. Key migration impacts:

| Change | Affects |
|--------|---------|
| `Bundle` types deprecated → required components | rocket.rs, ground.rs, main.rs (camera, environment) |
| bevy_xpbd_3d → avian3d | physics.rs, rocket.rs, ground.rs, main.rs |
| `Camera3dBundle` removed | main.rs camera setup |
| Text/UI API overhaul | fps.rs, main.rs text setup |
| `Color::rgb()` → `Color::srgb()` | Throughout |
| bevy_egui version jump | main.rs UI |

This is the single biggest investment decision: migrate to 0.15 (touches every file) or stay on 0.13 and accept ecosystem isolation.

## Recommended Actions

### Quick wins (< 1 hour)
1. Delete dead code: CCD stubs, `_notes.rs` contents, unused `RocketState` fields, `FocalPoint`, dead terrain code, `locked_axes()` in rocket.rs
2. Replace `println!` with `info!`/`debug!` or remove
3. Fix `interpolate_to_target_alt` to actually use `delta_t`
4. Remove `TimedForces` HashSet (keep only `ForceTimer` component)

### Medium effort (few hours)
5. Extract input/UI/event systems from main.rs into modules
6. Standardize input handling on `ButtonInput<KeyCode>`
7. Use Bevy change detection instead of `flag_changed`
8. Share `MeshData` between cone.rs and fin.rs
9. Widen crash detection logic

### Major decision
10. Bevy 0.15 migration — significant effort, touches every file, but unlocks current ecosystem
