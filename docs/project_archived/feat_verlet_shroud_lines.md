---
priority: P2
---

# Verlet Shroud Lines + Canopy Rim Deformation

## Summary

Replace rigid shroud lines with Verlet-simulated strings that sag, go taut, and collapse naturally. Use their endpoints to drive canopy rim deformation, giving the parachute soft-body behavior without cloth simulation.

## Motivation

Shroud lines are currently rigid cylinders stretched between two points. They look like struts, not strings. On the ground they stay frozen in place. Making them flexible would:

- Look much more natural during descent (slight sag under gravity)
- Enable the canopy to deform asymmetrically on landing (strings go slack → rim droops)
- Open the door to wind-driven billowing later

## Approach: Verlet Particle Simulation

Each shroud line gets 1-2 intermediate particles (6 lines × 2 particles = 12 particles total). No physics engine involvement — just a custom system.

### Per-frame update

1. **Verlet integration**: `new_pos = pos + (pos - prev_pos) + gravity * dt²`
2. **Distance constraints** (2-3 iterations):
   - Pin bottom particle to DetachedCone base
   - Pin top particle: free to move, but constrained by segment max length
   - Enforce max distance between consecutive particles
3. **Render**: position cylinder segments between consecutive particles

### Canopy rim coupling

The key insight: the 6 shroud line top-endpoints **are** the canopy rim.

- Each shroud line's top particle defines a rim anchor point
- `animate_canopy_system` takes those 6 world-space points and deforms the SphericalCap mesh:
  - Rim vertices pin to their nearest anchor
  - Vertices between anchors interpolate between neighbors
  - Inner ring vertices interpolate inward toward cap center
  - Cap center = average of rim points + depth offset along average normal
- During descent: strings taut → rim roughly circular → canopy looks normal
- On landing: strings slack on one side → rim droops → canopy deforms asymmetrically

### Data model

```rust
#[derive(Component)]
struct ShroudLineParticles {
    positions: Vec<Vec3>,      // current positions (including pinned endpoints)
    prev_positions: Vec<Vec3>, // previous frame positions
    segment_length: f32,       // rest length per segment
}
```

Stored on each `ShroudLine` entity. At deploy time, spawn 2-3 child cylinder entities per line for segment rendering (replacing the current single cylinder).

## System scheduling

- `update_shroud_verlet_system` — Update schedule, after `animate_canopy_system` reads ParachuteBody transform
- Reads: DetachedCone transform (bottom pin), ParachuteBody transform (reference for rim)
- Writes: `ShroudLineParticles` positions
- `animate_canopy_system` then reads the 6 top-particle positions to deform the mesh

## Scope

- 6 shroud lines × 2 intermediate particles = 12 Verlet particles
- 6 × 3 = 18 cylinder segments (replacing current 6)
- Canopy mesh deformation driven by rim anchors
- ~50 lines of Verlet simulation code, ~30 lines of mesh deformation

## What this does NOT include

- Collision between strings and other objects
- String-to-string tangling
- Per-vertex canopy cloth simulation
- Wind forces on strings (but easy to add later — just apply wind to particles)

## Codex Review

### Findings

1. The proposal is currently frame-rate dependent in a way that conflicts with the project's Avian setup. The design describes a "per-frame update" for Verlet integration and constraints, but this project's rigid-body motion is owned by Avian in fixed-timestep systems. If the shroud solver runs in `Update` with variable `dt`, sag and slack will vary by frame rate and drift relative to `ParachuteBody`. If we proceed, the solver should run in `FixedUpdate` or `FixedPostUpdate` using `Time<Fixed>`, with visual transforms applied later in `PostUpdate`.
2. The scheduling section is internally inconsistent. It says `update_shroud_verlet_system` runs after `animate_canopy_system` reads `ParachuteBody`, but also says `animate_canopy_system` reads the top-particle positions from the shroud simulation. The dependency should be: solve shrouds first, then deform the canopy, then update the rendered line segments.
3. The design blurs a visual solver and a physical solver without choosing which one is authoritative. Right now Avian owns parachute rigid-body motion, tether correction, and drag, while custom systems own canopy visuals. The proposed free-moving shroud endpoints drive canopy shape, but nothing feeds that back into drag, tether tension, or the collider. That is acceptable only if this is explicitly treated as a visual secondary-motion system rather than as true parachute physics.
4. The canopy deformation description is under-constrained for the current mesh model. Pinning rim vertices to arbitrary world-space anchors and reconstructing the cap center from an averaged normal could invert triangles or twist the dome when one side goes slack. The current `SphericalCap` mesh assumes a clean radial ordering. Rim deformation should stay in canopy-local space with bounded offsets and clamped droop.
5. The proposed data model is workable but not an ideal Bevy ECS shape. `Vec<Vec3>` on every `ShroudLine` entity is unnecessary allocation for a fixed tiny particle count. Fixed-size arrays or a single rig component on the parachute entity would better match the static structure of this feature.
6. The proposal should include a validation plan. For a custom constraint solver, we should expect tests for segment-length conservation, endpoint pinning, and canopy mesh stability under bounded droop, similar to the targeted tests already used elsewhere in the parachute code.

### Assessment

The overall direction is reasonable. For this project scale, a small custom shroud solver is more Bevy-idiomatic than turning each line segment into a full physics object, and it fits the current split where Avian owns rigid-body physics while custom systems handle parachute visuals.

The main adjustment is scope. This should be framed as a visual shroud and canopy secondary-motion system, not as a new physical authority for the parachute. That keeps it consistent with the existing architecture and avoids conflicting with Avian's ownership of the rigid parachute body.

### Alternatives

1. Recommended: keep `ParachuteBody` as the physical authority, run a small fixed-step visual solver for the shroud particles, and deform the canopy from bounded local-space rim offsets. This gives the desired look without coupling visual sag directly into the rigid-body simulation.
2. Lower-complexity alternative: skip Verlet and use analytic sag per line, such as a simple Bezier or catenary-like curve driven by tension, descent speed, and ground contact. With only six lines, this may produce most of the visual gain at substantially lower implementation and maintenance cost.
3. Higher-fidelity alternative: if line tension and asymmetry should affect actual motion, use Avian joints or a custom XPBD constraint chain instead of a purely custom visual solver. Avian 0.5 supports joints and custom constraints, but that is a materially larger feature and likely not justified unless the parachute itself is moving toward a true soft-body simulation.

## Claude Review

### Agreement with Codex

Findings #1 (frame-rate dependence), #2 (scheduling inconsistency), and #3 (visual vs physical authority) are the most important catches. The Verlet solver must run in `FixedUpdate` with `Time<Fixed>`, the dependency order is shrouds → canopy → segments, and this should be explicitly scoped as visual secondary motion.

Findings #4 (mesh inversion risk) and #5 (Vec vs fixed arrays) are valid but less critical — bounded droop in local space handles #4, and #5 is a minor cleanup at implementation time. #6 (tests) is standard good practice.

### Recommendation: Start with Analytic Sag (Alternative 2)

Codex's Alternative 2 deserves to be the v1 implementation. With only 6 lines, analytic sag gives ~80% of the visual payoff at a fraction of the complexity:

- Compute a tension parameter per line from `actual_distance / rest_length`
- Blend between straight (taut, tension ≥ 1) and catenary-like sag (slack, tension < 1)
- Render each line as 2-3 cylinder segments along the curve
- On ground contact: endpoint distance shrinks → tension drops → lines visibly sag

No constraint solver, no iteration, no fixed-timestep requirements. The visual difference between a catenary approximation and a 2-particle Verlet chain is negligible at this scale.

Verlet (the original proposal) becomes worth revisiting if we later want wind-driven billowing or line tangling — but that's a v2 concern.

### Proposed v1 Scope

1. Add a `ShroudLineSag` component with rest length and sag curve parameters
2. System computes per-line tension from endpoint distance vs rest length
3. Render each line as 2-3 segments along an analytic sag curve (quadratic or catenary)
4. On landing: lines naturally go slack as endpoints converge
5. Tests for sag computation (taut → straight, slack → bounded droop)
