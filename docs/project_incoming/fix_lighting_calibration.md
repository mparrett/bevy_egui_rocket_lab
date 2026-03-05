# Calibrate lighting for cubemap and atmosphere modes

## Problem

Lighting is not well-calibrated across the project:
- **Atmosphere mode, night**: pitch black even with AutoExposure
- **Atmosphere mode, day**: slightly too bright
- **Cubemap mode**: looks acceptable after switching to default Exposure + brightness 1000.0, but hasn't been deeply reviewed
- **Indoor scenes (Lab, Store)**: too dark after removing `Exposure::BLENDER` — room lighting (point lights, spot lights) needs rebalancing for default exposure

The lighting setup evolved organically and needs a coherent pass against Bevy's reference examples.

## Current state

- **Cubemap mode**: default Exposure (no explicit component), `Skybox { brightness: 1000.0 }`, `GlobalAmbientLight` animated 0.25–0.5 based on time of day
- **Atmosphere mode**: `AutoExposure::default()`, `DirectionalLight` illuminance ramps from `FULL_MOON_NIGHT` to `RAW_SUNLIGHT`, `GlobalAmbientLight` animated 0.55–1.05
- **Tonemapping**: `TonyMcMapface` (Bevy atmosphere example uses `AcesFitted`)
- **Store scene**: `Exposure::BLENDER` removed, uses default

## Reference examples

- Skybox: https://github.com/bevyengine/bevy/blob/latest/examples/3d/skybox.rs
  - `Skybox { brightness: 1000.0 }`, `GlobalAmbientLight { brightness: 1.0 }`, default Exposure
- Atmosphere: https://github.com/bevyengine/bevy/blob/latest/examples/3d/atmosphere.rs
  - `Exposure { ev100: 13.0 }`, `GlobalAmbientLight::NONE`, `illuminance: lux::RAW_SUNLIGHT`, `Tonemapping::AcesFitted`
- Auto exposure: https://github.com/bevyengine/bevy/blob/latest/examples/3d/auto_exposure.rs
  - `AutoExposure::default()`, `Skybox { brightness: lux::DIRECT_SUNLIGHT }`

## Areas to investigate

1. Why AutoExposure doesn't handle night properly — is scene too dark for it to adapt, or does it need tuning (min_ev, max_ev, compensation curve)?
2. Whether `GlobalAmbientLight` animation is helping or fighting with AutoExposure
3. Whether tonemapper choice matters (`TonyMcMapface` vs `AcesFitted`)
4. Cubemap mode brightness — systematic check rather than "it looks ok"
5. Day/night illuminance ramp — currently uses `CLEAR_SUNRISE` as floor for daylight > 0, `FULL_MOON_NIGHT` for night. Should this be smoother? Should cubemap and atmosphere modes share the same ramp?
6. Indoor scene lighting (Lab and Store) — point/spot light intensities were tuned for `Exposure::BLENDER` and need rebalancing for default exposure

## Goal

A lighting setup that:
- Looks good across the full day/night cycle in both modes
- Aligns with Bevy's recommended patterns
- Doesn't require bespoke tuning per skybox
