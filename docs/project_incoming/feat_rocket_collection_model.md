# Rocket Collection & Active Rocket Model

## Problem

The current system has a single `RocketDimensions` resource mutated directly by egui sliders. There's no concept of owning multiple rockets or selecting one. The rocket entity always exists — there's no "empty table" state. Saved blueprints (`RocketSave`) exist on disk but aren't loaded into a runtime collection.

This blocks several gameplay features:
- First-time play flow (player should start with no rocket, buy starter at Store)
- Multiple rocket designs the player can switch between
- Future: multiple rockets in a scene (e.g. NPC rockets, side-by-side comparison)

## Current State

- **One rocket entity**, spawned at Startup, repositioned per scene (table in Lab, pad in Launch, hidden in Store)
- **`RocketDimensions`** is a singleton resource — sliders mutate it directly
- **`RocketSave`** (name + dimensions + flight params) can be saved/loaded from disk
- **`SaveState.rocket_saves: Vec<String>`** tracks names but isn't a runtime collection
- **`EquippedLoadout`** (motor/parachute/tube/nosecone) is partially disconnected from dimensions
- Store's "Starter Rocket" creates a `RocketSave` on disk for $10

## Proposed Model

```rust
#[derive(Resource)]
pub struct RocketCollection {
    pub rockets: Vec<RocketSave>,
    pub active: Option<usize>,  // None = no rocket selected
}
```

### Key behaviors

- **`active: None`** means no rocket is in the scene. The rocket entity should be hidden or despawned. Lab shows empty table. Launch pad is empty (can't launch).
- **`active: Some(i)`** syncs `rockets[i].dimensions` → `RocketDimensions` resource, which drives the entity visuals as it does today.
- **Editing in Lab** mutates the active rocket's dimensions (through `RocketDimensions` as today, but synced back to the collection on changes).
- **Switching rockets** updates `active`, syncs dimensions, triggers `RocketGeometryChangedEvent`.
- **Creating a new rocket** pushes to the collection and sets it active.
- **Deleting a rocket** removes from collection; if it was active, set `active = None`.

### First-time play flow

1. Menu → Play (new player)
2. → Store (forced first visit), starter rocket available for purchase
3. Player buys starter → added to `RocketCollection`, set as active
4. → Lab, rocket appears on table, ready to customize and launch

### Future: multiple rockets in scene

The `RocketCollection` is player-owned inventory. Scene-level rocket spawning would be separate — each rocket entity references a `RocketSave` by index or ID. The single-entity pattern can evolve to multi-entity without changing the collection model. Consider giving `RocketSave` a stable UUID for entity references.

## Scope

### Phase 1 — Runtime collection + selection UI
- Add `RocketCollection` resource, populated from disk saves on player load
- Add rocket selector UI in Lab (dropdown or list)
- Sync active rocket ↔ `RocketDimensions`
- Handle `active: None` (hide rocket entity, disable launch)
- Wire Store "Starter Rocket" purchase to add to collection

### Phase 2 — First-time flow
- Detect new player (empty collection)
- Route Play → Store instead of Lab
- Guide player to buy starter rocket
- Then transition to Lab

### Phase 3 — Multi-entity (future)
- Scene can spawn multiple rocket entities from collection
- Each entity has its own dimensions, not the global resource
- Player's "active" rocket is the one they control

## Files likely affected

- `src/save.rs` — `RocketCollection` resource, load/save collection
- `src/rocket.rs` — sync active ↔ `RocketDimensions`, handle None state
- `src/main.rs` — Lab UI rocket selector, Store purchase wiring
- `src/scene.rs` — conditional rocket visibility based on `active`
- `src/inventory.rs` — possibly merge `EquippedLoadout` into `RocketSave`
