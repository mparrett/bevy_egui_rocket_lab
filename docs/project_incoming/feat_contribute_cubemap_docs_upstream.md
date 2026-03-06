---
priority: P4
---

# Contribute cubemap face ordering docs upstream to Bevy

## Context

We hit a cubemap face ordering bug in our skybox pipeline: py360convert outputs faces as F R B L U D, and the correct wgpu stacking order for Bevy's `reinterpret_stacked_2d_as_array` is R L U D F B (not R L U D B F as originally assumed). The +Z/-Z mapping between py360convert and wgpu is the opposite of what you'd naively expect.

This was debugged by analyzing edge pixel continuity between adjacent cubemap faces and cross-referencing with the Vulkan cubemap spec's face adjacency rules.

## Upstream issue

https://github.com/bevyengine/bevy/issues/19125 — "Importing cube maps is hard"

The issue documents the poor DX of getting HDRIs into Bevy as skybox cubemaps. Our `scripts/add-skybox.sh` is a working end-to-end pipeline (equirectangular HDR → stacked PNG + ASTC/ETC2 KTX2) that could be useful as:

1. A documented reference for the correct face ordering
2. A contribution to Bevy's cubemap tooling or docs
3. Input into bevy_cli's asset pipeline (https://github.com/TheBevyFlock/bevy_cli/issues/6)

## Possible contributions

- Comment on #19125 with our findings about the py360convert → wgpu face mapping
- PR to Bevy docs clarifying the expected stacked cubemap face order
- Share `add-skybox.sh` as a reference pipeline

## Priority

Low — do when we have a quiet moment. The upstream issue is active and people are working on better solutions (PR #19076 for equirectangular support).
