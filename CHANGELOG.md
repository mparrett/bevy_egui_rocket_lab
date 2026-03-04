# Changelog

## 0.2.0 — 2026-03-03

### Added
- Sun orbit day/night cycle with configurable time-of-day and speed
- HUD clock readout (HH:MM) in the score bar
- Time and day-speed sliders in the Sky panel
- Spring-based camera follow system with second-order tracking
- Opt-in volumetric fog toggle (GPU heavy)
- Mouse orbit camera (left-click drag, scroll wheel zoom)
- Multiple skybox options with runtime switching (Grasslands Sunset, Belfast Sunset, Citrus Orchard, Bambanani Sunset)
- Per-skybox atmospheric fog colors
- Mipmapped KTX2 ground texture for reduced aliasing
- Shared fin mesh/material handles for efficiency
- Particle emitter re-anchoring on rocket geometry changes
- ECS regression test suite (10 tests covering launch/reset/landing core loop)
- GitHub Pages auto-deploy workflow for WASM builds
- Loading overlay to hide asset pop-in
- WASM build support (dev and release profiles, wasm-opt)

### Changed
- Upgraded Bevy 0.13.1 → 0.18 with full API migration
- Upgraded physics from bevy_xpbd_3d 0.4.2 to avian3d 0.5
- Thrust synced with fixed physics tick for consistent behavior
- Camera target tracking runs after physics writeback
- Separated camera distance and truck keybinds
- Remapped slowmo key; Space/Enter both trigger launch
- Split native and WASM release profiles for better optimization
- Skybox upgraded from 256px to 512px cube faces

### Fixed
- egui sliders no longer interfere with camera orbit drag
- Camera follow lag reduced via tighter spring parameters
- Camera state re-seeded on follow mode switches to prevent jumps
- Anisotropic filtering for ground texture quality
- Fog UI simplified to single combo box
- Text layout and scoreboard positioning fixes
- Physics force direction corrected

## 0.1.0

Initial release — Bevy-based model rocketry sandbox with egui controls, basic physics, and skybox rendering.
