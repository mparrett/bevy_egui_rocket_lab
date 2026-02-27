# Key Facts

## Versions

- Bevy: 0.13.1
- bevy_egui: 0.25.0
- bevy_xpbd_3d: 0.4.2
- bevy-inspector-egui: 0.23.2
- bevy_firework: git rev a191fd8
- bevy_generative: 0.2.0
- Rust edition: 2021

## Build

- Dev profile: opt-level=1 for crate, opt-level=3 for deps
- WASM target: wasm32-unknown-unknown + wasm-bindgen
- Clippy allows: type_complexity, too_many_arguments

## Assets

- Skybox cubemap: `textures/Ryfjallet_cubemap_astc4x4.ktx2` (index 2)
- Ground texture: `textures/GroundGrassGreen002_COL_4K_1024.png.ktx2`
- Audio: `audio/Welcome_to_the_Lab_v1.ogg` (loop), `air-rushes-out-fast-long.ogg` (launch), `impact_wood.ogg` (crash)
- Font: `fonts/FiraMono-Medium.ttf`
- Texture conversion scripts: `scripts/`

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
