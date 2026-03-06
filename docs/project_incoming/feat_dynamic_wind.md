# Dynamic Wind

## Summary

Add a dynamic wind system that applies lateral forces to the rocket during flight, simulating real atmospheric conditions. Wind direction and strength should vary over time (non-stationary), and be adjustable via the UI.

## Motivation

Real rockets deal with wind shear, gusts, and shifting wind patterns. Adding dynamic wind makes flights less predictable and more realistic, and sets the stage for future stability/control challenges.

## Behavior

- **Strength**: User-adjustable via slider in the egui panel (0 = calm, max = strong gusts)
- **Direction**: 2D vector (horizontal plane) that drifts over time using smooth noise (e.g. Perlin/simplex or filtered random walk) — wind shouldn't snap or teleport
- **Non-stationary**: Base direction and strength should wander slowly, with faster small perturbations layered on top (gusts)
- **Force application**: Applied as an external force to the rocket entity while in flight, scaled by wind strength

## UI

- Slider for wind strength (0.0–1.0)
- Wind direction indicator — a simple vector/arrow widget in the panel, like a clock hand showing current wind direction and relative magnitude
- Could reuse egui's `paint` API for a small circular widget with a line/arrow

## Implementation notes

- The existing `ForceTimer` system in `physics.rs` applies timed forces — wind should be a continuous system instead, applying force every frame while the rocket is launched
- Wind resource: `WindProperties { strength: f32, direction: Vec2, ... }` with internal noise state
- System runs in `Update` with `run_if(in_state(AppState::Launch))`
- Force applied via avian3d's `ExternalForce` equivalent (check avian3d 0.5 API — may need `Forces` QueryData)

## Open questions

- Should wind affect the rocket only, or also particle effects (smoke trail)?
- Altitude-dependent wind profiles (stronger wind at higher altitude)?
- Persist wind settings across sessions?

## Priority

Medium — enhances realism and gameplay feel. Low complexity given existing force infrastructure.
