---
priority: P2
---

# Bug: WASM Audio Blocked by Browser Autoplay Policy

## Status: TODO

## Problem
Browsers block audio playback until the user interacts with the page (click, tap, keypress). The game spawns background music and sound effects immediately, which will be silently suppressed on WASM until the first user gesture.

## Possible Approaches
- **Click-to-start overlay**: Add a "Click to Play" screen before entering the game loop. This is the standard pattern for web games.
- **Resume AudioContext on first interaction**: Use a JS event listener to resume the audio context after any user gesture, no extra UI needed. Bevy's web audio backend may need a nudge from JS.
- **Defer audio spawn**: Don't spawn the music player until after a Bevy input event is detected (e.g., first keypress). Simplest Rust-only approach but the user might miss the music start.

## Notes
- Native builds are unaffected
- The loading overlay already requires the user to wait — if we add a click-to-start, it could replace or follow the loading overlay
