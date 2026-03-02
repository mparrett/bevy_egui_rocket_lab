# Bug: Fog only affects ground plane, looks bad

## Status: TODO

## Problem

DistanceFog applies uniformly based on camera distance, so it fades the ground plane into a flat white/blue band at the horizon. It doesn't affect the skybox at all, creating a harsh seam between the fogged ground and the un-fogged sky. The result looks more like a rendering artifact than atmospheric haze.

Screenshots show the issue clearly at medium camera distance — a bright white band sits between the green ground and the skybox treeline.

## Current behavior

- Fog mode 0 (default): `DistanceFog::default()` — gray fog, visible immediately
- Fog mode 1: atmospheric visibility-based fog with directional light scattering
- Fog mode 2: linear fog (start=80, end=160)
- All modes only affect geometry (ground plane, rocket), not the skybox

## Desired behavior

Atmospheric fog that blends smoothly into the skybox at the horizon. Options to investigate:

1. **Match fog color to skybox horizon color** — sample the dominant horizon color from each skybox and set fog color accordingly. Cheap but per-skybox.
2. **Bevy's `AtmosphericFog`** — if available in 0.18, this may handle sky-ground blending natively.
3. **Increase ground plane size + push fog further out** — reduce the visible seam by making the ground extend beyond fog range.
4. **Disable fog entirely** — simplest fix if none of the above look good. (Done as interim: fog starts with alpha=0.)

## Interim fix

Fog disabled by default (alpha=0 on spawn). Still togglable with F/T keys for experimentation.
