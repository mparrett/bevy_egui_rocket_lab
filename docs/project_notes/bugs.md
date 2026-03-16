# Bug Log

## Skybox cubemap face ordering / seams

**Status:** Open
**Severity:** Visual — major seams visible at face boundaries
**Observed:** 2026-03-03

The skybox cubemaps have visible hard seams where faces meet. Not subtle filtering artifacts — the adjacent faces appear to be mismatched (wrong order or orientation). Observed in the "Belfast" skybox. The default "Grasslands Sunset" skybox looks fine, so this may be per-skybox rather than systemic.

**Likely cause:** Face ordering or orientation mismatch when the cubemap KTX2 files were generated. Different tools expect different face orders (OpenGL vs Vulkan conventions).

**Next steps:**
1. Check how the source equirectangular/cross images were split into cubemap faces
2. Verify face order matches what Bevy/wgpu expects (+X, -X, +Y, -Y, +Z, -Z)
3. Re-encode KTX2 cubemaps with correct face layout
4. Test all skybox variants, not just the default

## Camera follow semantics: "Above" can dip below target / "Side" lacks dynamic orbit frame

**Status:** Open
**Severity:** UX polish — medium
**Observed:** 2026-03-03

Current chase camera behavior is improved, but naming/intent still diverges from feel:
- `FollowAbove` (now `CameraViewpoint::FollowAbove`) should bias to stay above the rocket and avoid under-looking shots.
- `FollowSide` (now `CameraViewpoint::FollowSide`) is currently a fixed offset; it does not support dynamic side framing around rocket heading/orbit intent.

**Next steps:**
1. Define explicit constraints for `FollowAbove` (min relative height, max under-angle)
2. Add a rocket-relative side frame so `FollowSide` can orbit around heading rather than world-axis lock
3. Expose small tuning knobs in camera config (above bias strength, side orbit aggressiveness)
4. Add a focused camera-mode transition test plan (Ground→Side, Ground→Above, Side↔Above during ascent)

## Atmosphere mode incompatible with bevy_firework particle pipeline

**Status:** Workaround in place (atmosphere disabled during launch)
**Severity:** Crash — wgpu validation error
**Observed:** 2026-03-05
**Upstream:** [bevyengine/bevy#21784](https://github.com/bevyengine/bevy/issues/21784)

When `Atmosphere` is on the camera, Bevy adds extra bind group entries (bindings 29-31) to `mesh_view_layout_multisampled_atmosphere`. bevy_firework 0.9's render pipeline is compiled against the non-atmosphere layout and panics on the mismatch.

**Workaround:** `enter_launch` forces `SkyRenderMode::Cubemap`, and the atmosphere option is greyed out in the launch UI. All atmosphere code paths are preserved for when the upstream fix lands.

**Resolution:** Wait for bevy_firework to support atmosphere bind group layouts, or for Bevy to decouple atmosphere bindings from the mesh view bind group.

## Particles invisible on WebGL2 — bevy_firework binding array incompatibility

**Status:** Open
**Severity:** Visual — major (no exhaust particles during flight on WebGL2)
**Observed:** 2026-03-14

bevy_firework 0.9 unconditionally uses `SetMeshViewBindingArrayBindGroup` and `view_layout.binding_array_layout` in its render pipeline descriptor (render.rs). WebGL2 does not support the `TEXTURE_BINDING_ARRAY` feature (confirmed by console warning: "Feature TEXTURE_BINDING_ARRAY is not supported on this device"). The particle render pipeline silently fails — particles are simulated on the CPU side but never rendered.

**Impact:** No exhaust flames, no smoke trail during rocket flight on the WebGL2 build. The rocket launches and flies normally but with no visual particle effects. Native and WebGPU builds are unaffected.

**Fix options:**
1. **Upstream fix in bevy_firework** — add fallback to `view_layout.main_layout` when `TEXTURE_BINDING_ARRAY` is unavailable (preferred)
2. **Fork bevy_firework** — patch render.rs to use non-binding-array layout on WebGL
3. **Alternative particle system** — replace bevy_firework with a WebGL2-compatible particle renderer

**Related:** The atmosphere bind group incompatibility bug (above) is also a bevy_firework pipeline layout issue — both stem from bevy_firework's render pipeline being tightly coupled to a specific mesh view bind group layout.

## Multi-camera viewport rendering broken on WebGL2 (glow backend)

**Status:** Upstream limitation — documented
**Severity:** Feature loss — PiP cameras non-functional on WebGL2
**Observed:** 2026-03-13
**Upstream:** [bevyengine/bevy#17167](https://github.com/bevyengine/bevy/issues/17167), [bevyengine/bevy#17375](https://github.com/bevyengine/bevy/issues/17375)

A second camera with `Camera { order: 1, viewport: Some(...) }` produces no visible output on the glow/WebGL2 backend. Tested with `is_active: true` from spawn, a hardcoded viewport, and `ClearColorConfig::Custom(Color::RED)` — nothing renders. No console errors or panics; the failure is completely silent. The main camera shows through as if the second camera doesn't exist.

The same setup works correctly on native (wgpu Metal/Vulkan) and WebGPU WASM builds.

**Impact:** PiP cameras (drone cam, rocket cam) are non-functional on the WebGL2 build. The V-key toggle and all aux-cam systems still compile and run, but the second camera's output is never visible.

**Workaround:** Accepted as a known limitation of the WebGL2 build. PiP is a nice-to-have, not essential. The feature remains functional on native and WebGPU builds. On WebGL2, RocketCam is still usable as a main camera viewpoint (cycle with the mode button), which provides the same rocket-perspective view without requiring PiP.
