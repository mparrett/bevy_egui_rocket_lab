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
