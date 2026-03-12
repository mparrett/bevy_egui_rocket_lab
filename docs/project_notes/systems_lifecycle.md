# Physics Systems Lifecycle & State Transitions

Reference for how physics systems behave across scene and state transitions.

## State Overview

**AppState** (scene): `Menu` → `Lab` / `Launch` / `Store`
**RocketStateEnum** (flight): `Initial` → `Launched` → `Descending` → `Grounded`

## System Registration Table

### Force Systems (FixedPostUpdate, PhysicsSystems::First)

| System | File | Run Condition | Target Entity | Internal Guard |
|--------|------|---------------|---------------|----------------|
| `update_forces_system` | physics.rs | `in_state(Launch)` | Rocket (ForceTimer) | Timer finished → remove ForceTimer |
| `apply_wind_force_system` | wind.rs | `in_state(Launch)` | Rocket | speed_sq < 1e-6 early return |
| `apply_aerodynamic_drag_system` | drag.rs | `in_state(Launch)` | Rocket | state ∈ {Launched, Descending} |
| `apply_cone_drag_system` | drag.rs | `in_state(Launch)` | DetachedCone | query fails if none spawned |
| `parachute_drag_system` | parachute.rs | `in_state(Launch)` | ParachuteBody | `deployed == false` early return |

### Event/Message Systems (Update)

| System | File | Run Condition | Trigger | Ordering |
|--------|------|---------------|---------|----------|
| `on_launch_event` | main.rs | `in_state(Launch)` | LaunchEvent | — |
| `on_reset_event` | main.rs | `in_gameplay` | ResetEvent | `.after(cleanup_parachute_system)` |
| `auto_deploy_parachute_system` | parachute.rs | `in_state(Launch)` | per-frame poll | — |
| `deploy_parachute_system` | parachute.rs | `in_state(Launch)` | DeployParachuteEvent | — |
| `cleanup_parachute_system` | parachute.rs | `in_state(Launch)` | ResetEvent | before on_reset_event |
| `detect_landing_from_collision_system` | main.rs | `in_state(Launch)` | CollisionStart | — |
| `on_crash_event` | main.rs | `in_state(Launch)` | DownedEvent | — |

### Visual Tracking Systems (PostUpdate)

| System | File | Run Condition | Ordering |
|--------|------|---------------|----------|
| `rocket_position_system` | main.rs | `in_gameplay` | after Writeback, before TransformPropagate |
| `update_camera_transform_system` | camera.rs | `in_gameplay` | chained after rocket_position_system |
| `update_shock_cord_system` | parachute.rs | `in_gameplay` | after Writeback |
| `update_shroud_lines_system` | parachute.rs | `in_gameplay` | after Writeback |

## Scene Transition Matrix

| Transition | Physics Reset | Parachute Cleanup | Rocket Repositioned | Axes Locked | Notes |
|------------|---------------|-------------------|---------------------|-------------|-------|
| Menu → Lab | Yes (enter_indoor) | No (nothing to clean) | Table height | Yes | — |
| Menu → Launch | Yes (enter_launch) | No (nothing to clean) | Ground level | Yes | Forces sky to Cubemap |
| Lab → Launch | Yes (enter_launch) | No (nothing to clean) | Ground level | Yes | — |
| Launch → Lab | Yes (enter_indoor) | Yes (OnExit cleanup) | Table height | Yes | — |
| Launch → Store | Yes (enter_indoor) | Yes (OnExit cleanup) | Table height, hidden | Yes | — |
| Lab → Store | Yes (enter_indoor) | No (nothing to clean) | Table height, hidden | Yes | — |
| Store → Lab | Yes (enter_indoor) | No (nothing to clean) | Table height | Yes | — |
| Any (R key) | Yes (on_reset_event) | Yes (cleanup_parachute) | Per-scene | Yes | — |

### What scene enter handlers do

**enter_indoor** (shared by Lab & Store):
- Hides OutdoorMarker entities
- Resets rocket: position = table, rotation = identity, velocity = zero, axes = locked
- Sets RocketStateEnum::Initial
- Does NOT touch parachute entities/config

**enter_launch**:
- Shows OutdoorMarker entities
- Resets rocket: position = ground level, velocity = zero, axes = locked
- Sets RocketStateEnum::Initial
- Does NOT touch parachute entities/config

**OnExit(Launch)** — `disable_aux_cams_on_exit` + `cleanup_parachute_on_scene_exit` + `save_camera_on_exit`:
- Disables rocket cam and drone cam
- Despawns parachute entities if deployed, resets config
- Saves camera snapshot

## RocketStateEnum Transition Map

```
Initial ──[LaunchEvent]──► Launched ──[DeployParachuteEvent]──► Descending
   ▲                          │                                      │
   │                          │ [CollisionStart + v.y ≤ 0.25]       │ [CollisionStart + v.y ≤ 0.25]
   │                          ▼                                      ▼
   └───────[ResetEvent]───── Grounded ◄──────────────────────────────┘
```

### What each transition does

| Transition | System | Actions |
|------------|--------|---------|
| Initial → Launched | on_launch_event | Unlock axes, add ForceTimer, add EjectionTimer |
| Launched → Descending | deploy_parachute_system | Spawn detached cone + parachute body + joints, hide original cone, reparent cam |
| Launched/Descending → Grounded | detect_landing + on_crash_event | Set state, collapse canopy, increase damping |
| Any → Initial | cleanup_parachute + on_reset_event | Despawn recovery entities, lock axes, zero velocity, reset position |

## Entity Lifecycle

### Persistent entities (survive all transitions)

| Entity | Components | Notes |
|--------|------------|-------|
| Rocket | RocketMarker, RigidBody::Dynamic, Collider, LockedAxes | Position/velocity reset, axes re-locked |
| RocketCone (child) | RocketCone, Collider | Hidden during deploy, restored on reset |
| Fins (children) | FinMarker | Rebuilt when dimensions change |
| Main camera | Camera3d | Always present |
| Rocket cam | RocketCamMarker, Camera | Reparented between cone and detached cone |
| Drone cam | DroneCamMarker, Camera | Position reset on scene change |
| Ground | Collider | Always present |

### Ephemeral entities (spawn on deploy, despawn on reset)

| Entity | Spawned By | Despawned By | Components |
|--------|-----------|--------------|------------|
| DetachedCone | deploy_parachute_system | cleanup_parachute_system | RigidBody::Dynamic, Collider, Mass |
| ParachuteBody | deploy_parachute_system | cleanup_parachute_system | RigidBody::Dynamic, Collider |
| ShockCord (visual) | deploy_parachute_system | cleanup_parachute_system | Mesh3d only |
| ParachuteVisual | deploy_parachute_system | cleanup_parachute_system | Mesh3d, CanopyAnimation |
| ShroudLine ×18 | deploy_parachute_system | cleanup_parachute_system | Mesh3d only |
| ShockCordJoint | deploy_parachute_system | cleanup_parachute_system | DistanceJoint, RecoveryJoint |
| ShroudLineJoint | deploy_parachute_system | cleanup_parachute_system | DistanceJoint, RecoveryJoint |

### Ephemeral components (added/removed on rocket entity)

| Component | Added By | Removed By |
|-----------|----------|------------|
| ForceTimer | on_launch_event | update_forces_system (timer done) OR on_reset_event |
| EjectionTimer | on_launch_event | auto_deploy_parachute_system OR cleanup_parachute_system |

## Resolved Issues

### Parachute entities leaked on scene transition (FIXED)

Scene enter handlers reset the rocket but didn't despawn parachute entities. Fixed by adding `cleanup_parachute_on_scene_exit` to `OnExit(AppState::Launch)`. The cleanup logic is shared with `cleanup_parachute_system` via the `do_parachute_cleanup` helper.

### Camera reparenting race (FIXED)

System ordering between `cleanup_parachute_system` and `on_reset_event` was non-deterministic. Fixed by adding `.after(parachute::cleanup_parachute_system)` to `on_reset_event`.

## Minor Notes

### ForceTimer removed in two places

`update_forces_system` removes it when timer finishes. `on_reset_event` also removes it. Harmless (double-remove is a no-op) but indicates unclear ownership.
