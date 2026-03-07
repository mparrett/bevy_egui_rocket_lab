---
priority: P2
---

# Explicit rocket mass model

## Summary

Replace the current collider-density-based rocket mass with an explicit part-based mass model. Keep the existing colliders for contact/landing, but compute mass, center of mass, and angular inertia from rocket parts and materials instead of treating the body and nose as solid shapes.

## Motivation

The current rocket mass is derived from collider geometry plus `ColliderDensity`, which implies:

- Solid cylindrical body instead of a hollow tube
- Solid cone instead of a thin nose shell
- No fin mass contribution
- No motor/payload mass contribution
- No clean path to future propellant burn and center-of-mass shift

That is good enough for a placeholder rigid body, but not for model rockets.

## Goal

Move to a mass model that can represent:

- Hollow body tubes
- Nose cone shells
- Fins with their own material mass
- Motor mass near the tail
- A future split between dry mass and propellant mass

## Phase 1

Implement a default explicit mass model with fixed assumptions:

- Body tube shell mass from outer radius, wall thickness, and body material density
- Nose cone shell mass from cone radius, slant height, shell thickness, and nose material density
- Fin mass from fin geometry, fin thickness, and fin material density
- Motor represented as a simple tail-mounted lumped mass

Apply the aggregate result to the rocket rigid body using explicit `Mass`, `AngularInertia`, and `CenterOfMass`, and disable automatic collider-derived mass for the rigid body.

## Phase 2

Expose the mass-model assumptions to the tuning/save system:

- Tube wall thickness
- Body material preset / density
- Nose material preset / density
- Fin material preset / density
- Motor selection / motor mass

Persist those values with rocket saves.

## Phase 3

Add time-varying motor mass / propellant burn:

- Dry mass vs propellant mass
- Center-of-mass shift during burn
- Optional thrust curve tied to motor selection

## Notes

- Collision geometry should stay simple and stable; it should not be the source of truth for mass.
- Phase 1 does not need perfect aerospace-grade inertia math. A stable, explicit approximation is enough as long as mass and CoM are no longer based on solid colliders.
