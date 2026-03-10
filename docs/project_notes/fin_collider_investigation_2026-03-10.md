# Fin Collider Investigation

**Date:** 2026-03-10

## Summary

The rocket currently has collision only for the body tube and nose cone. Fins are visual-only meshes attached as children of the rocket body. Adding fin colliders is feasible with avian3d 0.5, but the right approach depends on what problem we want to solve:

- Better ground contact and resting poses
- More realistic snagging/clipping against environment geometry
- Future aerodynamic work where fin planform should matter physically

For this project, the best near-term option is **one convex collider per fin as a child collider**, created from the fin geometry or from an explicit convex hull. This gives much better physical behavior than the current body+cone-only model without the instability and overhead risks of thin trimeshes.

## Current State

### Rocket collider setup today

- The rocket root is a dynamic rigid body with manual mass/inertia and child colliders for:
  - Body tube: `Collider::cylinder(...)`
  - Nose cone: `Collider::cone(...)`
- Fins are spawned as visual children only with `FinMarker`; they have no `Collider`.
- Runtime dimension edits rebuild fin visuals by despawning and respawning fin child entities.

Relevant code:

- `src/rocket.rs` spawns body and cone colliders, then spawns visual fins.
- `src/main.rs` rebuilds rocket geometry on dimension changes and currently removes/re-adds fin child entities.

### Important constraint from existing code

The parachute code already contains a warning that removing a child collider from an avian compound body caused AABB panics in practice. That means adding fin colliders is not just a collider-shape question; it also affects how we support runtime geometry edits.

## What avian3d supports

avian3d 0.5 supports all of the collider categories we would need:

- Primitive shapes such as `cuboid`, `capsule`, `cone`, `triangle`
- `compound(...)` colliders
- Mesh-derived colliders such as:
  - `convex_hull_from_mesh`
  - `convex_decomposition_from_mesh`
  - `trimesh_from_mesh`

Relevant avian guidance from local crate sources:

- Compound colliders are preferred over trimeshes for dynamic rigid bodies.
- Thin shapes like trimeshes benefit from `CollisionMargin`, but still tend to be less stable than convex shapes.

## Options

### Option 0: Keep current body+cone only

**Implementation cost:** none

**Pros**

- Fastest and most stable
- No new compound-body complexity

**Cons**

- Fins still pass through ground and scene geometry
- Grounded poses remain visibly wrong for tail strikes / fin contact
- No path toward fin-sensitive collision or aerodynamic work

### Option 1: Simple tail-envelope approximation

Approximate all fins together with one extra primitive collider near the tail, for example:

- One wider rear `cuboid`
- One short wide `cylinder`
- A small compound of 2-3 primitives forming a tail skirt

**Implementation cost:** low

**Pros**

- Cheapest real improvement
- Easy to keep stable
- No thin-shape issues

**Cons**

- Contact shape is obviously inaccurate
- Can overestimate fin collisions badly
- Poor fit if fin count/size changes significantly

**When to choose it**

- If the goal is only “rocket should not lie through its fins on the ground”

### Option 2: One primitive collider per fin

Give each fin a simple child collider such as a thin `cuboid` aligned to the fin transform.

**Implementation cost:** low to medium

**Pros**

- Better than a single tail envelope
- Cheap collision cost
- Easy to generate from current fin transforms

**Cons**

- A cuboid fits the rectangular bound, not the triangular prism shape
- Contacts near the sloped edge will still feel wrong
- Over-collides more than a proper hull

**When to choose it**

- If we want a fast first pass with low implementation risk

### Option 3: One convex collider per fin

Represent each fin as a convex child collider:

- Either build the convex hull explicitly from the 6 fin prism vertices
- Or derive it from the fin mesh using `Collider::convex_hull_from_mesh(...)`

**Implementation cost:** medium

**Pros**

- Best balance of accuracy, performance, and stability
- Matches the actual triangular-prism fin shape closely
- Works well as a dynamic compound body, which is what avian recommends
- Scales naturally with fin count and fin dimensions

**Cons**

- More work than primitive boxes
- Requires care in the runtime geometry-update path

**When to choose it**

- This is the recommended default if the goal is “real fin colliders” rather than a stopgap

### Option 4: Trimesh or convex decomposition from fin mesh

Generate the collider directly from the fin mesh:

- `trimesh_from_mesh(...)`
- `convex_decomposition_from_mesh(...)`

**Implementation cost:** medium to high

**Pros**

- Closest to render geometry
- Flexible if fin meshes become more complex later

**Cons**

- `trimesh` is a poor fit for thin dynamic shapes; hollow and more prone to stability problems
- Convex decomposition is overkill for the current fin mesh, which is already just a simple convex prism
- More CPU/setup complexity without clear gameplay benefit

**When to choose it**

- Only if fin geometry becomes materially more complex than the current triangular prism

## Recommendation

### Recommended near-term path

Implement **Option 3: one convex child collider per fin**.

Concretely:

1. Add a helper that returns the six local-space fin prism vertices from `Fin { height, length, width }`.
2. Build `Collider::convex_hull(points)` directly from those vertices.
3. Spawn each fin as a child with:
   - visual mesh/material/transform
   - convex collider
   - `CollisionLayers::new([GameLayer::Rocket], [GameLayer::Ground])`
   - `CollisionEventsEnabled` only if we actually need per-fin collision events
4. Keep manual mass/inertia on the rocket root as-is. Fin mass is already accounted for in `rocket_mass_properties`.

### Recommended fallback

If we want a cheaper first pass, use **Option 2 with thin cuboids** and treat that as an intermediate step only.

## Implementation Notes

### 1. Compound-body update safety matters more than shape generation

The current geometry refresh path in `src/main.rs` removes and re-adds fins. That is acceptable for visual-only children, but risky once fins also carry colliders.

Recommended adjustment:

- Only rebuild fin collider children while in non-flight states (`Lab` / `Store` / pre-launch), or
- Replace fin children in a controlled reset/recreate path for the full rocket entity, or
- Reuse existing fin child entities and mutate their mesh/collider/transform in place instead of despawning them

I would avoid “despawn collider children while live physics is running” unless we have a tested safe path for it.

### 2. Keep fin colliders as child colliders, not separate rigid bodies

The rocket should remain one rigid body with a compound collider. Separate rigid bodies with joints would be unnecessary complexity here.

### 3. No mass-property changes needed for phase 1

The rocket already uses manual mass, center of mass, and inertia. Because fin mass is already included analytically, adding fin colliders should **not** enable auto mass calculations.

### 4. Consider slightly lower restitution on fins if fin bounce looks silly

If fin-ground contact becomes too bouncy, tune friction/restitution on fin colliders independently from the body tube.

## Expected Impact

### What should improve

- Rocket should rest on fins more plausibly after landing
- Tail/fin strikes should register against ground and scene geometry
- Visual silhouette and contact shape should match better during tumbles

### What will not improve by itself

- Aerodynamic stability
- Weathercocking
- Fin-generated torque in airflow

Those require aerodynamic force modeling, not just colliders.

## Suggested Validation

1. Static contact test: rocket placed on ground with large fins should visibly rest on fin contacts rather than sinking through them.
2. Tail strike test: pitch rocket backward into ground and verify fin contact happens before deep body penetration.
3. Geometry-edit test: changing fin count/size in Lab should not panic or leave stale child colliders behind.
4. Reset/deploy regression test: parachute and rocket reset flows should still work with fin collider children present.

## Recommendation Summary

- **Do:** add one convex child collider per fin
- **Do:** preserve manual mass/inertia on the rocket root
- **Do:** treat geometry-update safety as a first-class part of the feature
- **Do not:** use dynamic trimesh fin colliders as the first implementation
- **Do not:** keep the current despawn/re-add fin path unchanged once fins carry colliders
