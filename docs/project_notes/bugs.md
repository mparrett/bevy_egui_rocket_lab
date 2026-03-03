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
