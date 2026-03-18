---
priority: P2
---

# Parachute Simulation

## Summary

Add a deployable parachute system that slows the rocket's descent after apogee, enabling soft landings and recovery.

## Motivation

Parachute deployment is a core part of model rocketry — most real launches use single or dual-deploy recovery systems. This adds a meaningful player decision (when to deploy) and a success condition beyond just going high.

## v1 — Deployment mechanics ✅ COMPLETE

Shipped across multiple commits (Feb–Mar 2026). What landed:

1. Deploy key (`P`) triggers chute while descending
2. Nose cone detaches as a separate avian3d physics body with upward impulse and collision layers
3. Procedural spherical cap canopy mesh (not placeholder) with inflation animation
4. Shroud lines with analytic Bezier sag from canopy rim to rocket body
5. Recovery tethers using avian3d `DistanceJoint` (cone↔rocket, canopy↔rocket)
6. Orientation-dependent v² aerodynamic drag (axial Cd derived from cone geometry)
7. Canopy collapses on landing
8. `RocketStateEnum::Descending` state, auto-deploy at apogee
9. Chute diameter slider in Lab panel
10. Full test coverage for joint spawning, cleanup, and lifecycle

### What v1 exceeded original scope on

- Spherical cap canopy mesh (was planned for v2)
- Shroud lines with sag (was planned for v2)
- Deployment animation (was planned for v2)
- Orientation-dependent drag (not originally scoped)

## v2 — Canopy secondary motion (next)

Canopy mesh and shroud lines already exist from v1. Remaining v2 work:

- Sinusoidal flutter / breathing animation on the canopy during descent
- Velocity-dependent canopy lag (tilts away from direction of travel)
- Wind interaction with canopy shape

## v3 — Cloth-like canopy

Verlet integration + distance constraints on a low-poly radial mesh (~49 verts: 12 segments × 4 rings). Parachute-specific dome/inflation bias forces. Proper wind interaction. Possibly bevy_silk integration. See design notes in conversation for detailed approach.

## References

- bevy_silk: https://github.com/ManevilleF/bevy_silk (v2/v3 candidate)
