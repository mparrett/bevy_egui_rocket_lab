---
priority: P2
---

# Feature: Random Rocket Explosion

## Summary

Add a random chance of the rocket exploding during flight, simulating real-world rocket failures (RUDs). Adds tension and unpredictability to launches.

## Details

- Small random chance of explosion during powered flight (e.g., engine failure, structural failure)
- Probability could scale with thrust, altitude, or flight duration
- Visual explosion effect (fireball, debris particles) using bevy_firework
- Sound effect on explosion
- Rocket transitions to `Grounded` state after explosion
- UI panel toggle to enable/disable random failures
- Adjustable failure probability slider in the egui panel

## Considerations

- Integrate with existing `RocketStateEnum` lifecycle (`Launched` → explosion → `Grounded`)
- Fire a `DownedEvent` (or new `ExplodedEvent`) so existing systems react appropriately
- Reuse/extend existing particle system for explosion visuals
- Keep it fun — failure rate should default to something low but noticeable
