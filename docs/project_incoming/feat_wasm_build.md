# Feature: WASM Build for Browser Testing

## Summary

Set up a working WASM build pipeline so the game can run in the browser. This enables faster iteration — just reload the tab instead of recompiling and relaunching natively.

## Context

- `just release-wasm` exists but hasn't been tested since the Bevy 0.13→0.18 upgrade
- Bevy 0.18 supports `wasm32-unknown-unknown` target
- wasm-bindgen-cli is required (see DEV.md)
- avian3d 0.5 should support WASM (verify)
- bevy_firework 0.9 should support WASM (verify)
- bevy_egui 0.39 supports WASM

## Tasks

1. **Verify/update `just release-wasm`** — ensure the justfile target works with current deps
2. **Fix any WASM-incompatible code** — e.g., audio codecs, file I/O, thread usage
3. **Add `index.html` harness** — minimal HTML page that loads the WASM module
4. **Add `just serve-wasm`** — local dev server (e.g., `python -m http.server` or `basic-http-server`)
5. **Test in browser** — verify rendering, egui panel, audio, physics all work
6. **Document** — update DEV.md with WASM build/serve instructions

## Known Risks

- Audio: OGG playback may need web audio API workarounds (user gesture to start)
- Texture loading: the PNG ground texture should work, but cubemap KTX2 may need fallback
- Performance: WASM builds are single-threaded; particle system may need tuning
- bevy_egui clipboard/input handling may differ on web

## Acceptance Criteria

- `just release-wasm && just serve-wasm` produces a working browser build
- Rocket renders, launches, and camera follows
- Egui panel is interactive
- No console errors on load
