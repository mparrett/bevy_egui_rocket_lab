# Decisions

## ADR-001: Pin bevy_egui to 0.25 (2024-03)

**Context**: bevy_egui 0.26 introduced a breaking issue.
**Decision**: Stay on 0.25 until upstream fix lands.
**Reference**: https://github.com/mvlabat/bevy_egui/issues/269
**Consequences**: Locked to Bevy 0.13.x ecosystem.

## ADR-002: Pin bevy_firework to git rev (2024-03)

**Context**: Particle system relies on an unreleased feature.
**Decision**: Pin to rev `a191fd8` from PR #12.
**Reference**: https://github.com/mbrea-c/bevy_firework/pull/12
**Consequences**: Cannot use crates.io version; must track upstream manually.

## ADR-003: Hybrid UI — egui for tuning, Bevy UI for gameplay (2026-03)

**Context**: Considered switching entirely to Bevy's native UI or entirely to egui.
**Decision**: Keep both. egui for the parameter tuning panel (sliders, collapsible sections, combo boxes). Bevy UI for in-game HUD elements (score, instructions, FPS counter — already using it).
**Rationale**: egui is far more concise for immediate-mode debug/tuning UI (~130 lines vs significantly more in Bevy UI). Bevy UI is better for player-facing elements (gamepad support, animations, consistent styling). Hide egui panel behind a toggle in "play" mode.
**Consequences**: Two UI dependencies. Accept the trade-off for developer velocity on the tuning panel.

## ADR-004: Use bevy_xpbd_3d for physics (2024-03) [superseded]

**Context**: Needed 3D rigid body physics for rocket flight simulation.
**Decision**: bevy_xpbd_3d 0.4.2.
**Consequences**: This crate was later absorbed into Bevy ecosystem as `avian3d`. Migration required on Bevy upgrade.
