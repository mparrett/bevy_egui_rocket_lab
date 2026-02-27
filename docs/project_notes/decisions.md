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

## ADR-003: Use bevy_xpbd_3d for physics (2024-03)

**Context**: Needed 3D rigid body physics for rocket flight simulation.
**Decision**: bevy_xpbd_3d 0.4.2.
**Consequences**: This crate was later absorbed into Bevy ecosystem as `avian3d`. Migration required on Bevy upgrade.
