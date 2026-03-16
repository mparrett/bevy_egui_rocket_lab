# Browser Testing with Playwright

## Setup
- Use `uv run --with=playwright python test-script.py` to run tests
- Must use `headless=False, channel='chrome'` for WebGL/WebGPU (headless Chromium uses SwiftShader which lacks GPU features)
- Set `device_scale_factor=2` on macOS Retina to match native DPR

## Bevy Canvas Coordinate Mapping (DPR=2)
- Bevy UI uses physical pixel coordinates internally (e.g. 2560x1734 on a 1280x867 CSS canvas)
- **CSS position = Bevy position / DPR**
- Viewport Y = (Bevy_Y / DPR) + HTML_top_bar_height (33px for our index.html)
- Example: Bevy `left: 296, top: 56` → CSS `(148, 61)` viewport

## Clicking Bevy UI Buttons on Canvas
Since Bevy UI renders to the canvas, Playwright can't find buttons by selector.
Must click by computed pixel coordinates.

### Title Screen (Menu state)
- Layout: full-screen centered column, `row_gap: 40px` (Bevy physical px)
- Items (WASM): Title (72px font) → Play → Settings → Quit
- Button height: ~60px Bevy (28px font + 32px padding + border)
- Play button center (CSS): `(canvas_width/2, canvas_top + (canvas_height - total_height/DPR) / 2 + (72+40+30)/DPR)`
- Approximately: `(640, cy + 411)` for 1280x900 viewport with DPR=2

### Navigation Between States
- **Keyboard:** `Tab` key cycles Lab → Launch → Store (most reliable method!)
- Nav buttons ("← Shop", "Launch →") are Bevy UI at `left: 296, top: 56` (Bevy physical px)
- After converting: CSS ≈ `(148, 61)` viewport — but hard to hit precisely

### Launch State
- "Enter/Space: launch" triggers countdown
- Click canvas center-right area first (away from egui panel) to ensure Bevy input focus
- Then press Enter or Space

### Triggering the Launch
1. Click Play on title screen
2. Press Tab to switch from Lab to Launch
3. Click canvas (right side, away from egui panel) for focus
4. Press Enter to start countdown → rocket launches after 3-2-1

## Gotchas
- egui captures keyboard input when its panel has focus — click the 3D viewport area first
- WebGL2 SwiftShader is very slow (~2-5 FPS), allow generous wait times (20-25s for load)
- The `Autofocus processing was blocked` console message is expected
- Tab key is intercepted by Bevy for state cycling, but only after canvas has focus
