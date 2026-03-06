---
priority: P1
---

# Bug: Physics Feel Off After avian3d Migration

## Summary

After migrating from bevy_xpbd_3d 0.4.2 to avian3d 0.5, the physics behavior doesn't feel right. Needs investigation.

## Possible Causes

1. **Force magnitude scaling** — avian3d may use different units or scaling for `Forces::apply_force()` vs old `ExternalForce::apply_force()`. Default force (0.2 N) may need retuning.
2. **Collider parameter order** — avian3d swapped `Collider::cylinder(radius, height)` from xpbd's `(half_height, radius)`. Verify the rocket's mass/volume is correct.
3. **Mass computation** — `ColliderDensity(1.0)` on child colliders; verify avian3d computes compound mass the same way.
4. **Damping** — `LinearDamping(0.4)` and `AngularDamping(1.0)` may behave differently in avian3d.
5. **Gravity** — Using `Gravity(Vec3::NEG_Y * 9.81)`. Verify this is the correct resource for avian3d 0.5.
6. **ForceTimer system** — `Forces` QueryData applies forces for one frame only (not persistent). Verify the timer system is applying forces every frame during the burn, not just once.

## Steps to Investigate

- Add debug logging for force magnitude, rocket mass, and velocity each frame during launch
- Compare flight trajectory (max height, flight time) to expected values for the given force/mass/duration
- Check if `Forces::apply_force()` accumulates correctly when called every frame
- Try increasing force to 2.0 or 5.0 to see if the rocket moves more dramatically
