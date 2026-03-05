# Fix skybox face ordering + add Learner Park skybox

## Problem

Non-default skyboxes (Belfast, Citrus Orchard, Bambanani) have visible hard seams at cubemap face boundaries. The default Grasslands Sunset looks fine. See `docs/project_notes/bugs.md` for details.

## Root cause (likely)

The broken skyboxes were probably converted manually or with an older script before `scripts/add-skybox.sh` was written. The current `add-skybox.sh` has correct wgpu face ordering (`R L U D B F` mapped from py360convert's `F R B L U D`), so re-running it on the broken skyboxes should fix them.

## Plan

1. **Re-generate broken skyboxes** using `add-skybox.sh`:
   ```sh
   ./scripts/add-skybox.sh belfast_sunset
   ./scripts/add-skybox.sh citrus_orchard
   ./scripts/add-skybox.sh bambanani_sunset
   ```
2. **Verify** each skybox in-app — check all face boundaries for seams
3. **If faces are still wrong**, the py360convert face order assumptions on line 42 of `add-skybox.sh` need investigation. Test with a known-good reference HDRI.
4. **Add Learner Park** skybox for the Store scene:
   ```sh
   ./scripts/add-skybox.sh learner_park
   ```
5. **Wire into code**: add `SkyboxEntry` to `SKYBOXES` in `src/sky.rs` with appropriate fog colors (soft sunrise, cool tones)
6. **Optionally**: use Learner Park as the Store scene's visible-through-window skybox (future enhancement — store currently uses indoor room with no skybox switching)

## Dependencies

- `ktx` CLI tool (KTX-Software)
- `toktx` CLI tool (KTX-Software)
- `uv` with py360convert, opencv-python-headless, Pillow, numpy

## References

- Learner Park HDRI: https://polyhaven.com/a/learner_park (CC0, rooftop parking at sunrise)
- Existing bug: `docs/project_notes/bugs.md` — "Skybox cubemap face ordering / seams"
- Feature ticket: `docs/project_incoming/feat_skybox_and_webgpu.md`
