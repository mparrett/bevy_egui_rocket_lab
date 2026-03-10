# Firework Fix WIP Review

**Status:** Open  
**Reviewed:** 2026-03-03  
**Scope:** local WIP in `bevy-rocket-lab` plus local `bevy_firework` override at `/tmp/bevy_firework`

## Findings

### 1) [P1] `Cargo.toml` now depends on an absolute `/tmp` path

**Where:** `Cargo.toml:11`  
**Details:** `bevy_firework = { path = "/tmp/bevy_firework" }` makes builds non-reproducible for teammates and CI, and also makes this branch environment-specific.  
**Impact:** Anyone without that exact local path cannot build this branch.  
**Recommendation:** Replace with a shareable dependency source (`crates.io`, git rev, workspace-relative path, or `[patch.crates-io]` committed in-repo).

### 2) [P1] `Q -> Menu` introduces duplicate HUD entities and breaks score text updates after re-entering Play

**Where:**
- `src/main.rs:309-312` (`Q` now sets `AppState::Menu`)
- `src/main.rs:162` (`setup_text_system` runs on every `OnEnter(AppState::Playing)`)
- `src/main.rs:1060-1078` (new `ScoreMarker` entity is spawned each enter)
- `src/main.rs:588` (`text_query.single_mut()` early-returns when multiple `ScoreMarker` entities exist)

**Details:** After going `Playing -> Menu -> Playing`, text entities are spawned again with no matching cleanup on exit from `Playing`. Once there are 2+ `ScoreMarker` entities, `single_mut()` fails and the stats text stops updating.  
**Recommendation:** Mark play-HUD entities with `DespawnOnExit(AppState::Playing)` (or add an explicit `OnExit(AppState::Playing)` cleanup system), and optionally make the score query robust to duplicates.

## Firework Fix-Specific Note

The local `bevy_firework` changes in `/tmp/bevy_firework/src/core.rs` and `/tmp/bevy_firework/src/render.rs` look directionally correct for decoupling queueing from `RenderMeshInstances` (using extracted center data instead), which should avoid dropping firework draw calls for non-mesh spawner entities.

## Validation Run

- `CARGO_TARGET_DIR=/tmp/rocket-lab-check cargo check`  
  Result: **pass** (includes `/tmp/bevy_firework` and `bevy-rocket-lab`)
