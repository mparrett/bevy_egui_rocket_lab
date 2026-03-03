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
- `FollowAbove` should bias to stay above the rocket and avoid under-looking shots.
- `FollowSide` is currently a fixed offset; it does not support dynamic side framing around rocket heading/orbit intent.

**Next steps:**
1. Define explicit constraints for `FollowAbove` (min relative height, max under-angle)
2. Add a rocket-relative side frame so `FollowSide` can orbit around heading rather than world-axis lock
3. Expose small tuning knobs in camera config (above bias strength, side orbit aggressiveness)
4. Add a focused camera-mode transition test plan (Ground→Side, Ground→Above, Side↔Above during ascent)
