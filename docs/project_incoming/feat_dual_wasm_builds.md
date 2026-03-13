---
priority: P3
---

# Dual WASM Builds (WebGPU + WebGL2)

## Summary

Ship two separate WASM builds — a WebGL2 compatibility build (especially for iOS) and a WebGPU build (desktop Chromium, future Safari) — with a JS loader that picks the right one at runtime.

## Motivation

Current deployment is WebGPU-only (decided 2026-03-02, see archived `feat_skybox_and_webgpu.md`). That works for desktop browsers but excludes iOS Safari, which won't ship WebGPU until Safari 26. A friend researched the state of play in Bevy 0.18 and confirmed:

- `bevy/webgl2` and `bevy/webgpu` are separate feature paths; one WASM binary cannot do both (upstream: bevyengine/bevy#13168, bevyengine/bevy#11505)
- iOS Safari 17.4–18.7 has WebGPU disabled by default; Safari 26 beta is the first with it enabled
- The practical answer for now is two builds with a JS-side router

## Approach

### 1. Cargo features for build selection

```toml
[features]
default = []
web_webgl = ["bevy/webgl2"]
web_webgpu = ["bevy/webgpu"]
```

### 2. Separate build targets in justfile

Two new just targets (e.g. `just release-wasm-webgl`, `just release-wasm-webgpu`) producing separate `.js`/`.wasm` artifacts.

### 3. JS loader with iOS detection

A small `<script type="module">` block that checks `navigator.gpu` and UA string, loads the WebGPU build for desktop Chromium and the WebGL2 build for iOS / fallback.

### 4. Feature-gate WebGPU-only code paths

Use `#[cfg(feature = "web_webgpu")]` / `#[cfg(feature = "web_webgl")]` for any backend-specific rendering. Keep gameplay and shared logic backend-agnostic.

## Scoping (assessed 2026-03-13)

### WebGPU-only features that need gating

**Critical — must disable for WebGL2:**
- Bloom (all 3 cameras in `main.rs`)
- HDR / `Hdr` component (all 3 cameras)
- TonyMcMapface tonemapping (all 3 cameras) — use a simpler tonemapper
- Volumetric fog (`sky.rs`) — already has a runtime `volumetrics_enabled` toggle
- HDR emissive values up to 22,000.0 (`sky.rs`, `rocket.rs`, `parachute.rs`, `particles.rs`, `scene.rs`) — clamp to [0-1]

**Moderate — may need adjustment:**
- Cascade shadows (`sky.rs`, `scene.rs`) — may need simpler shadow config
- bevy_firework (`particles.rs`) — unknown WebGL2 compat, needs investigation

**Already handled:**
- ASTC/ETC2 textures — `sky.rs` already falls back to PNG via `RenderDevice::features()`

### Build infrastructure changes

- **Cargo.toml**: `bevy/webgpu` is currently hardcoded; make it feature-conditional
- **justfile**: add `release-wasm-webgl` / `dev-wasm-webgl` targets with separate output dirs
- **index.html**: JS loader to detect `navigator.gpu` + iOS UA and pick the right artifact
- **CI** (`.github/workflows/ci.yml`): build and deploy both artifacts to GitHub Pages

### What's already backend-agnostic (no changes needed)

Gameplay, physics (avian3d), parachute system, UI (egui), save/load, menu, input handling — all clean.

### Effort estimate

- Build plumbing (Cargo features, justfile, index.html, CI): ~1 day
- Feature-gate rendering (Bloom/HDR/volumetrics/tonemapping/emissives): ~1 day
- Testing WebGL2 build in Safari + mobile: ~half day
- bevy_firework investigation: unknown (could be zero, could require a fallback particle system)

## Design constraints

- WebGL2 build must avoid compute shaders, storage-buffer-heavy techniques, and other WebGPU-only features
- Content/shaders should target a lowest-common-denominator where possible so both builds behave similarly
- Expect a WASM size increase (two artifacts instead of one); keep 1K textures for size budget

## References

- bevyengine/bevy#13168 — Support WebGL2 and WebGPU in the same WASM file (open)
- bevyengine/bevy#11505 — Support both WebGPU and WebGL2 in the same wasm binary (open)
- https://caniuse.com/webgpu — browser support matrix
- Bevy cargo features docs: https://github.com/bevyengine/bevy/blob/main/docs/cargo_features.md

## Origin

Researched by a friend (external recommendation, Mar 2026). Raw notes in `webgpu-incoming.txt`.
