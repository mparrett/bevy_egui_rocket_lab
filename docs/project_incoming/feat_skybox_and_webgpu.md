# Feature: Skybox Selection & WebGPU Support

## Status: IN PROGRESS

## Tasks

### 1. Swap in grasslands_sunset skybox — DONE
Replaced Ryfjallet with Poly Haven "grasslands_sunset" HDRI. ASTC/ETC2/PNG fallback chain in place.

### 2. Add selectable skybox locations
Add UI to let the user choose between multiple skyboxes at runtime:
- **grasslands_sunset** — golden grassland sunset (default)
- **belfast_sunset** — pastel hilltop evening (https://polyhaven.com/a/belfast_sunset)
- **citrus_orchard** — soft sunrise over orchard (https://polyhaven.com/a/citrus_orchard)
- **bambanani_sunset** — warm golden-hour dry landscape (https://polyhaven.com/a/bambanani_sunset)

Each needs: download 1K HDR, convert to cubemap (ASTC + ETC2 + PNG), add to assets, wire into egui panel.

### 3. WebGPU — DONE (already active)
Investigated 2026-03-02. Findings:
- **Already using WebGPU.** With no `webgl2` feature in Cargo.toml, wgpu 27 defaults to WebGPU backend for WASM.
- **No code or HTML changes needed.** Current `index.html` works for both backends.
- **Browser support is strong** (Chrome 113+, Firefox 141+, Safari 26). Covers all current desktop browsers.
- **Dual-backend fallback (WebGPU → WebGL2) is not yet possible** in Bevy 0.18. Tracked upstream: bevyengine/bevy#13168.
- **Decision: Stay WebGPU-only.** This is a tech demo targeting desktop browsers. Revisit dual-backend when Bevy ships #13168.
- Texture fallback (ASTC/ETC2/PNG) works correctly under WebGPU — format availability is GPU-dependent, not API-dependent.

## Notes
- Keep 1K resolution for WASM size budget (~33MB optimized currently)
- All four HDRIs are CC0 from Poly Haven
- Cubemap conversion pipeline: likely `hdri_to_cubemap` or similar Rust/CLI tool, or Bevy's asset pipeline
