---
priority: P3
---

# Rocket-Cam POV Mode

## Summary

Add a first-person camera mode mounted on the rocket, giving an onboard view during flight — like real rocket onboard footage.

## Motivation

Onboard cameras are a staple of real rocketry streams. This mode will be especially compelling once dynamic wind and physical perturbations are in place — feeling the rocket wobble and correct from a nose-cam perspective.

## Behavior

- Camera position: attached to the rocket entity (child transform), likely near the nose cone looking outward
- Look direction: probably downward-angled or configurable — straight up (sky view) is less interesting than seeing the ground fall away
- Should track rocket orientation naturally (tilts, spins, wobble all visible)
- Consider a slight offset or gimbal stabilization option for comfort

## Integration

- Add as a new variant in `CAMERA_MODES` / `FollowMode` enum in `camera.rs`
- Cycle into it with the existing camera mode toggle
- May need to hide the rocket mesh (or make it transparent) to avoid clipping

## Open questions

- Fixed look direction (e.g. 30° below horizontal) vs free-look from the rocket's frame?
- Multiple mount points (nose-cam, fin-cam, side-cam)?
- HUD overlay with altitude/velocity while in this mode?

## Priority

Low — fun feature, best after wind/physics improvements land.
