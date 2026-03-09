---
priority: P2
---

# Parachute Simulation

## Summary

Add a deployable parachute system that slows the rocket's descent after apogee, enabling soft landings and recovery.

## Motivation

Parachute deployment is a core part of model rocketry — most real launches use single or dual-deploy recovery systems. This adds a meaningful player decision (when to deploy) and a success condition beyond just going high.

## v1 — Deployment mechanics (current focus)

Focus on the deployment flow and drag physics. Visual is a placeholder (ball gizmo on a cord, or simple streamer). The deployment sequence mirrors real model rockets:

1. Player presses deploy key (e.g. `P`) while rocket is descending
2. Nose cone pops off — becomes a separate rigid body with small upward impulse
3. Shock cord visual (thin cylinder) connects cone to tube
4. Placeholder chute visual spawns above tube (ball gizmo or streamer)
5. Drag force applies to rocket: `F = 0.5 * Cd * A * rho * v²` opposing velocity
6. Rocket descends at terminal velocity, lands softly

### Rocket lifecycle

- New state: `RocketStateEnum::Descending` (chute deployed) between `Launched` and `Grounded`
- Deploy only valid when rocket is above a minimum altitude and has been launched
- Landing with chute deployed = soft landing; without = crash

### UI

- Deploy keybind (`P`)
- Chute diameter slider in Lab panel (affects drag area `A`)
- Status indicator: "Chute: stowed / deployed / landed"

### What v1 does NOT include

- Proper canopy mesh (ball/streamer placeholder only)
- Cloth simulation
- Wind interaction with chute
- Tangling / failure modes
- Dual deploy

## v2 — Spherical cap canopy

Replace placeholder with a procedural spherical cap mesh. Add secondary motion via shape relaxation (inflation parameter, sinusoidal flutter, velocity lag). Shroud lines as thin cylinders from rim to tube. State-driven deployment animation (packed → deploying → inflating → open).

## v3 — Cloth-like canopy

Verlet integration + distance constraints on a low-poly radial mesh (~49 verts: 12 segments × 4 rings). Parachute-specific dome/inflation bias forces. Proper wind interaction. Possibly bevy_silk integration. See design notes in conversation for detailed approach.

## References

- bevy_silk: https://github.com/ManevilleF/bevy_silk (v2/v3 candidate)
