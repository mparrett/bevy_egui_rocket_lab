# GUI Focus Management Investigation

**Date:** 2026-03-10

## Summary

There is a focus handoff bug between egui and gameplay input when switching into `Launch` mode with `Tab`.

Observed behavior:

- Press `Tab` to move from `Lab` to `Launch`
- The app enters `Launch` state correctly
- Pressing `Enter` does not launch until the user clicks back into the 3D view

The most likely cause is that egui keeps keyboard focus from the previously active UI control, and the gameplay hotkey system treats **any** egui keyboard focus as a reason to ignore launch input.

## Current Input Flow

### State switching

`init_egui_ui_input_system` handles `Tab` first and can switch app state even while egui has keyboard focus.

Relevant behavior:

- `Tab` is handled before the `ctx.wants_keyboard_input()` early return
- So `Lab -> Launch` transition works even if a slider / combo box / text field in egui still owns focus

### Launch hotkey

`do_launch_system` blocks all launch/camera hotkeys whenever:

- `ctx.wants_keyboard_input()` returns `true`

That includes:

- `Enter`
- `Space`
- camera controls (`C`, `Z`, arrows)

Because focus is not cleared on transition, `Launch` mode begins with egui still owning keyboard input, so the launch hotkey path never runs until the user clicks outside the panel.

## Findings

### 1. Focus is preserved across app-state transitions

Switching from `Lab` to `Launch` does not clear the previously focused egui widget. This is usually desirable within a single UI mode, but in this case it conflicts with gameplay controls that are expected to become active immediately after mode switch.

### 2. `wants_keyboard_input()` is too coarse for gameplay gating

`ctx.wants_keyboard_input()` is a broad signal: it means egui would like keyboard input, not necessarily that a text-edit field is actively consuming `Enter` for text entry.

Using it as a blanket blocker for gameplay input makes the app overly sticky once any egui control is focused.

### 3. The issue likely affects more than launch

Because the same gating pattern is used elsewhere, this probably also suppresses:

- reset hotkey behavior (`R`)
- quit hotkey behavior (`Q`)
- camera mode/zoom/orbit hotkeys in `Launch`

depending on which egui widget last held focus.

## Likely Root Cause

The bug is not “Enter launch is broken.” The bug is:

1. `Tab` transitions into `Launch`
2. egui focus is retained from the previous panel interaction
3. `do_launch_system` sees `ctx.wants_keyboard_input()`
4. gameplay input returns early
5. clicking the 3D render causes egui to relinquish focus
6. `Enter` starts working again

## Recommended Fixes

### Recommended fix: clear egui focus on transition into `Launch`

When `Tab` changes app state to `Launch`, explicitly clear egui keyboard focus / active text-edit state.

That gives the cleanest UX:

- `Tab` into `Launch`
- gameplay keys work immediately
- user does not need an extra click

This should be done at the transition boundary, not left to incidental pointer behavior.

### Recommended supporting fix: narrow the gameplay input block

Avoid using `ctx.wants_keyboard_input()` as the only gate for gameplay hotkeys in `Launch`.

Instead, suppress gameplay controls only when:

- a text-entry widget is active, or
- a specific focused egui control should legitimately consume the key

For example:

- `Enter` should probably still launch if focus is sitting on a slider or collapsed section
- `Enter` should probably not launch if a text field is actively being edited

This is especially relevant because the current UI only has a small number of true text-entry contexts.

### Optional fix: add explicit launch button / focus indicator

As a UX improvement, consider one of:

- a visible `Launch` button in the egui panel
- a small HUD hint showing whether gameplay controls or UI currently own keyboard focus

This is not the core fix, but it would make focus issues easier to diagnose in the future.

## Suggested Implementation Plan

### Phase 1

1. On `Lab -> Launch` transition, clear egui keyboard focus
2. Verify `Enter` launches immediately after pressing `Tab`
3. Verify `C`, `Z`, arrow controls also work immediately in `Launch`

### Phase 2

1. Audit other uses of `ctx.wants_keyboard_input()`
2. Replace broad gating with narrower conditions where appropriate
3. Keep text-entry widgets protected from gameplay hotkeys

## Validation Checklist

1. Interact with a slider in `Lab`, then press `Tab`, then `Enter`
   Expected: launch happens immediately
2. Interact with a combo box in `Lab`, then press `Tab`, then `C`
   Expected: camera mode cycles immediately
3. Focus a real text field (for example rocket save name), then switch modes
   Expected: text focus is not left in a state that blocks gameplay unexpectedly
4. Switch back and forth repeatedly between `Lab` and `Launch`
   Expected: no sticky focus requiring viewport clicks

## Recommendation Summary

- **Do:** treat this as a focus-transition bug, not a launch-action bug
- **Do:** clear egui focus when entering `Launch`
- **Do:** narrow `wants_keyboard_input()`-based blocking in gameplay systems
- **Do not:** rely on “click the 3D view” as the focus handoff mechanism
