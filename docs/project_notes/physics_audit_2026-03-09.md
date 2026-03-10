# Rocket Flight Physics Audit

**Date:** 2026-03-09

## Overview

Audit of all physical systems affecting rocket flight. Engine: avian3d 0.5 on Bevy 0.18, fixed-timestep physics in `FixedPostUpdate`.

## Systems Inventory

### Force Application (all in `PhysicsSystems::First`)

| System | File | Description |
|--------|------|-------------|
| `update_forces_system` | `physics.rs:56-79` | Timed thrust via `ForceTimer`. Local or world-space force/torque with timer expiration. |
| `apply_wind_force_system` | `wind.rs:114-147` | Wind force at variable center-of-pressure point along body. Procedural 3-frequency gusting. |
| `parachute_drag_system` | `parachute.rs` | Standard drag equation (Cd=0.8, rho=1.225) on deployed chute body. Capped at 50N. |

Recovery chain (Rocket ↔ DetachedCone ↔ ParachuteBody) connected via avian3d `DistanceJoint` constraints. Shock cord joint: max 1.0m. Shroud line joint: max `parachute_config.diameter`. Forces transmit naturally through the solver.

### Launch & Landing (in `Update`)

| System | File | Description |
|--------|------|-------------|
| `on_launch_event` | `main.rs:579-626` | Creates ForceTimer (default 4N for 1s), unlocks axes, starts ejection timer. |
| `detect_landing_from_collision_system` | `main.rs:700-728` | Reads `CollisionStart`, fires `DownedEvent` if velocity.y <= 0.25 m/s. |
| `on_reset_event` | `main.rs:645-698` | Zeros velocities, re-locks axes, resets position. |

### Mass Model (`rocket.rs:354-401`)

Manual mass properties (`NoAutoMass`, `NoAutoAngularInertia`, `NoAutoCenterOfMass`). Composed of body shell (hollow cylinder), nose cone shell, motor (point mass at base), and fins (triangular). Parallel axis theorem for composite inertia. CoM always below geometric center for passive stability.

### Rocket Body (`rocket.rs:456-546`)

`RigidBody::Dynamic`, `LinearDamping(0.4)`, `AngularDamping(1.0)`. Child colliders for body cylinder and nose cone (Friction 0.7, Restitution 0.4). `LockedAxes::all()` until launch. `TransformInterpolation` for smooth rendering.

## Constants Reference

| Param | Value | Location |
|-------|-------|----------|
| Gravity | 9.81 m/s² | `main.rs:184` |
| Default thrust | 4.0 N / 1.0 s | `rocket.rs` (RocketFlightParameters) |
| Linear damping | 0.4 | `rocket.rs` |
| Angular damping | 1.0 | `rocket.rs` |
| Wind axial coeff | 0.0005 | `wind.rs` |
| Wind lateral coeff | 0.0012 | `wind.rs` |
| Max wind force | 0.05 N | `wind.rs` |
| Max wind speed | 8.0 m/s horiz, 2.5 m/s vert | `wind.rs` |
| Parachute Cd | 0.8 | `parachute.rs` |
| Parachute diameter | 0.3 m (default) | `parachute.rs` |
| Air density (rho) | 1.225 kg/m³ | `parachute.rs` |
| Max drag force | 50.0 N | `parachute.rs` |
| Ejection delay | 3.0 s post-burnout | `parachute.rs` |
| Tether length (shock cord) | 1.0 m | `parachute.rs` |
| Joint damping (linear) | 2.0 | `parachute.rs` |
| Landing velocity threshold | 0.25 m/s | `main.rs` |
| Rocket friction / restitution | 0.7 / 0.4 | `rocket.rs` |
| Ground friction / restitution | 0.7 / 0.2 | `ground.rs` |

## Findings

### P1: No crash detection for high-speed impacts

**Location:** `detect_landing_from_collision_system` (`main.rs:700-728`)

The only landing trigger requires `velocity.y <= 0.25 m/s`. A rocket hitting the ground at 5+ m/s doesn't register — it bounces (restitution 0.4) until slow enough. There's no separate crash path, so hard impacts look wrong: the rocket bounces around with no consequence.

**Impact:** Visible gameplay issue. Crashes should look and feel like crashes.

### P2: Linear damping used as air drag proxy

**Status:** Resolved — `LinearDamping(0.0)` now set on rocket entity. Proper v² drag via `apply_aerodynamic_drag_system` in `drag.rs` with axial/lateral decomposition and geometry-derived Cd.

### P3: No aerodynamic stability model

**Location:** `wind.rs` — CP offset is randomized, not geometry-based

Real rockets are stable when center-of-pressure (CP) is behind center-of-gravity (CG). The wind system randomizes CP position along the body instead of deriving it from geometry. Consequences:
- A rocket with fins (should be stable) and one without fins (should be unstable) behave similarly
- No natural weathercocking or divergence
- Fin size/count has no aerodynamic effect, only mass effect

### P4: Detached cone has hardcoded gravity

**Status:** Resolved — cone is now `RigidBody::Dynamic` with avian3d `DistanceJoint` tethers, uses engine gravity automatically.

### P5: Wind CP offset discontinuity

**Location:** `wind.rs` — `cp_offset_y_norm` retargets every 0.5-1.4s

When the center-of-pressure offset jumps to a new target, it creates a sudden torque change. On rockets with low angular inertia, this could cause visible jitter.

**Impact:** Minor — smoothing the transition would help.

### P6: Damping active on ground

**Location:** `rocket.rs` — LinearDamping/AngularDamping always active

Damping fights the friction model while grounded. Probably imperceptible but technically incorrect.

**Impact:** Negligible.

### P7: Constant air density

Air density is hardcoded at sea level (1.225 kg/m³). Model rockets don't go high enough for this to matter in practice.

**Impact:** Negligible for current scale.

## What's Working Well

- Manual mass/inertia with parallel axis theorem — solid, physically correct
- Parachute drag equation — textbook implementation
- Tether spring constraint — simple, stable, good feel
- Force-at-point wind model — good foundation even with randomized CP
- Physics scheduling — forces in `First`, visuals in `PostUpdate`, no race conditions
- Numerical guards — 50N drag cap, 1e-6 speed threshold, axis locking
