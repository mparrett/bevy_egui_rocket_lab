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

## ADR-005: avian3d physics integration patterns (2026-03)

**Context**: Repeated stutter/jitter issues when integrating custom physics logic (forces, constraints, visual tracking) with avian3d's fixed-timestep simulation.
**Decision**: Follow these rules for all physics-related systems:

1. **Forces/impulses** go in `FixedPostUpdate` / `PhysicsSystems::First` — applied before the sim step.
2. **Post-sim constraints** (tethers, clamps, position corrections) go in `FixedPostUpdate` after `PhysicsSystems::StepSimulation` and before `PhysicsSystems::Writeback`. Modify `Position`/`Rotation` directly (not `Transform`) so the corrected state feeds into Writeback and interpolation.
3. **Any dynamic rigid body that the camera or visual systems track** must have `TransformInterpolation` for smooth rendering between fixed steps.
4. **Systems reading physics state for visuals** (camera follow, cord/line positioning) run in `PostUpdate` after `PhysicsSystems::Writeback`. If one system writes a value another reads (e.g. rocket position → camera target), chain them explicitly.
5. **Collision layers** separate entity groups that should never interact (e.g. Rocket vs Debris) to prevent compound-body overlap impulses.

**Rationale**: avian3d's pipeline is First → Prepare → StepSimulation → Writeback. Modifying `Transform` in First gets synced to `Position` in Prepare but then overwritten by StepSimulation. Constraints must run post-sim. Without `TransformInterpolation`, Transforms update discretely at the fixed rate causing visible stutter. Without explicit system ordering in PostUpdate, Bevy runs systems in arbitrary order causing frame-to-frame jitter.

## ADR-006: WebGPU-only for WASM (2026-03) [superseded]

**Context**: Bevy 0.18 / wgpu 27 defaults to WebGPU for WASM builds. Dual-backend (WebGPU + WebGL2 fallback) in a single binary is not yet supported (bevyengine/bevy#13168).
**Decision**: Stay WebGPU-only. All current desktop browsers support it (Chrome 113+, Firefox 141+, Safari 26).
**Rationale**: This is a tech demo targeting desktop browsers. Adding a JS shim + dual builds for WebGL2 fallback isn't worth the complexity.
**Consequences**: Older browsers and some mobile devices won't work. Revisit when Bevy ships #13168.
**Superseded**: By ADR-007 (dual WASM builds).

## ADR-007: Dual WASM builds — WebGPU + WebGL2 (2026-03)

**Context**: iOS Safari disables WebGPU until Safari 26 (late 2026). The single WebGPU-only build (ADR-006) excludes all iOS users.
**Decision**: Ship two separate WASM binaries — one compiled with `--features web_webgpu` and one with `--features web_webgl`. A JS loader in index.html detects WebGPU support and dynamically imports the correct build.
**Rationale**: Bevy doesn't support dual backends in a single binary (bevyengine/bevy#13168). Two separate builds with Cargo features is the cleanest approach. The WebGL2 build disables HDR (Bloom, volumetric fog, high emissive values) via `cfg` gates since WebGL2 lacks the required framebuffer precision.
**WebGL2 differences**: No Bloom, no volumetric fog, Reinhard tonemapping instead of TonyMcMapface, emissive values clamped to 1.0, skybox brightness 1.0 instead of 1000.0.
**Consequences**: CI builds take ~2x longer. Two output directories (`out-webgpu/`, `out-webgl/`). Deploy artifact is larger. Native builds are unaffected (no web features enabled).
