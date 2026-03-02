# Key Facts

## Versions

- Bevy: 0.18
- bevy_egui: 0.39
- bevy-inspector-egui: 0.36
- avian3d: 0.5 (formerly bevy_xpbd_3d)
- bevy_firework: 0.9
- bevy_utilitarian: 0.9
- Rust edition: 2024

## Build

- Dev profile: opt-level=1 for crate, opt-level=3 for deps
- Release profile: opt-level='z', lto=true, codegen-units=1, strip=true, panic="abort"
- WASM target: wasm32-unknown-unknown + wasm-bindgen
- WASM backend: WebGPU (wgpu 27 default, no `webgl2` feature)
- WASM binary size: ~33MB after wasm-opt -Oz
- Clippy allows: type_complexity, too_many_arguments
- Auto-deploys to GitHub Pages on push to main

## Assets

- Skybox cubemap: `textures/grasslands_sunset_cubemap_astc4x4.ktx2` (ASTC), with ETC2 and PNG fallbacks
- Ground texture: `textures/GroundGrassGreen002_COL_4K_1024.png.ktx2`
- Audio: `audio/Welcome_to_the_Lab_v1.ogg` (loop), `air-rushes-out-fast-long.ogg` (launch), `impact_wood.ogg` (crash)
- Font: `fonts/FiraMono-Medium.ttf`

## Controls

- Enter: Launch
- R: Reset
- Q: Quit
- C: Cycle camera mode
- Z: Cycle zoom
- D/S: Destabilize/Stabilize
- F: Toggle fog
- T: Cycle fog type
- L: Toggle fog lighting
- Space: Toggle slow-mo
- Arrow keys: Camera orbit/distance
- Escape: World inspector
- F12: FPS counter
