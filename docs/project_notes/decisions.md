# Decisions

## ADR-001: Pin bevy_egui to 0.25 (2024-03) [superseded]

**Context**: bevy_egui 0.26 introduced a breaking issue.
**Decision**: Stay on 0.25 until upstream fix lands.
**Reference**: https://github.com/mvlabat/bevy_egui/issues/269
**Superseded**: Upgraded to bevy_egui 0.39 with Bevy 0.18 migration (2026-02).

## ADR-002: Pin bevy_firework to git rev (2024-03) [superseded]

**Context**: Particle system relies on an unreleased feature.
**Decision**: Pin to rev `a191fd8` from PR #12.
**Reference**: https://github.com/mbrea-c/bevy_firework/pull/12
**Superseded**: Upgraded to bevy_firework 0.9 from crates.io with Bevy 0.18 migration (2026-02).

## ADR-003: Hybrid UI — egui for tuning, Bevy UI for gameplay (2026-03)

**Context**: Considered switching entirely to Bevy's native UI or entirely to egui.
**Decision**: Keep both. egui for the parameter tuning panel (sliders, collapsible sections, combo boxes). Bevy UI for in-game HUD elements (score, instructions, FPS counter — already using it).
**Rationale**: egui is far more concise for immediate-mode debug/tuning UI (~130 lines vs significantly more in Bevy UI). Bevy UI is better for player-facing elements (gamepad support, animations, consistent styling). Hide egui panel behind a toggle in "play" mode.
**Consequences**: Two UI dependencies. Accept the trade-off for developer velocity on the tuning panel.

## ADR-004: Use bevy_xpbd_3d for physics (2024-03) [superseded]

**Context**: Needed 3D rigid body physics for rocket flight simulation.
**Decision**: bevy_xpbd_3d 0.4.2.
**Superseded**: Migrated to avian3d 0.5 (bevy_xpbd_3d's successor) with Bevy 0.18 migration (2026-02).

## ADR-005: WebGPU-only for WASM (2026-03)

**Context**: Bevy 0.18 / wgpu 27 defaults to WebGPU for WASM builds. Dual-backend (WebGPU + WebGL2 fallback) in a single binary is not yet supported (bevyengine/bevy#13168).
**Decision**: Stay WebGPU-only. All current desktop browsers support it (Chrome 113+, Firefox 141+, Safari 26).
**Rationale**: This is a tech demo targeting desktop browsers. Adding a JS shim + dual builds for WebGL2 fallback isn't worth the complexity.
**Consequences**: Older browsers and some mobile devices won't work. Revisit when Bevy ships #13168.
