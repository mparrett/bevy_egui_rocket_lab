# Parachute Simulation

## Summary

Add a deployable parachute system that slows the rocket's descent after apogee, enabling soft landings and recovery.

## Motivation

Parachute deployment is a core part of model rocketry — most real launches use single or dual-deploy recovery systems. This adds a meaningful player decision (when to deploy) and a success condition beyond just going high.

## Behavior

- **Deploy trigger**: Player-activated (keypress) or automatic at apogee / below a configurable altitude
- **Drag model**: Once deployed, apply an upward drag force proportional to descent velocity squared (simplified parachute drag: `F = 0.5 * Cd * A * rho * v^2`)
- **Visual**: Spawn a parachute mesh (cone/hemisphere + lines) as a child entity above the rocket, billowing effect optional
- **Descent rate**: Should settle to a realistic terminal velocity based on chute size — tunable via UI

## Rocket lifecycle integration

- New state: `RocketStateEnum::Descending` (chute deployed) between `Launched` and `Grounded`
- Deploy only valid when rocket is above a minimum altitude and descending (negative vertical velocity)
- Landing with chute deployed = soft landing; without = crash (tie into future launch history scoring)

## UI

- Deploy button or keybind (e.g. `P` or `Space` after apogee)
- Chute diameter slider in the Lab panel (affects drag area)
- Status indicator: "Chute: stowed / deployed / landed"

## Open questions

- Single deploy vs dual deploy (drogue at apogee, main at low altitude)?
- Should the chute be a physics object with its own collider, or just a force modifier on the rocket?
- Tangling / failure modes for added challenge?
- How does wind (if implemented) interact with the chute?

## Priority

Medium-high — central to the model rocketry experience and natural next step for the flight lifecycle.
