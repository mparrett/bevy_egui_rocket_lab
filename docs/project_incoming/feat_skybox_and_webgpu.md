# Feature: Skybox Selection & WebGPU Support

## Status: TODO

## Tasks

### 1. Swap in grasslands_sunset skybox
Replace the current Ryfjallet sample cubemap with the Poly Haven "grasslands_sunset" HDRI.
- Download 1K HDR from https://polyhaven.com/a/grasslands_sunset
- Convert equirectangular HDR → cubemap KTX2 (ASTC 4x4 or ETC2, ensure dimensions are block-size multiples)
- Update `src/sky.rs` CUBEMAPS array and asset path
- Verify on desktop and WASM

### 2. Add selectable skybox locations
Add UI to let the user choose between multiple skyboxes at runtime:
- **grasslands_sunset** — golden grassland sunset (default)
- **belfast_sunset** — pastel hilltop evening (https://polyhaven.com/a/belfast_sunset)
- **citrus_orchard** — soft sunrise over orchard (https://polyhaven.com/a/citrus_orchard)
- **bambanani_sunset** — warm golden-hour dry landscape (https://polyhaven.com/a/bambanani_sunset)

Each needs: download 1K HDR, convert to cubemap, add to assets, wire into egui panel or Bevy UI selector.

### 3. Enable WebGPU
Switch WASM build to support WebGPU (with WebGL2 fallback).
- Bevy 0.18 supports `wgpu` backends — check if a feature flag or runtime detection is needed
- Update `index.html` if WebGPU init differs from WebGL2
- Test in Chrome (WebGPU enabled) and Safari/Firefox (WebGL2 fallback)
- Document any browser requirements

## Notes
- Keep 1K resolution for WASM size budget (~37MB optimized currently)
- All four HDRIs are CC0 from Poly Haven
- Cubemap conversion pipeline: likely `hdri_to_cubemap` or similar Rust/CLI tool, or Bevy's asset pipeline
