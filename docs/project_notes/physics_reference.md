# Physics Reference

Physical modeling reference for the rocket flight simulator. All values in SI units (m, kg, s, N).

## Recovery Chain

After parachute deployment, three rigid bodies connected by avian3d `DistanceJoint` constraints:

```
Rocket ──shock cord (0–1m)──▶ DetachedCone ──shroud lines (0–diameter)──▶ ParachuteBody
```

- Drag applied to ParachuteBody → tension transmits through joints → decelerates rocket
- `JointDamping { linear: 1.0 }` on both joints to damp pendulum oscillations
- Cone and chute body on `GameLayer::Debris`, rocket on `GameLayer::Rocket` — prevents contact impulse double-counting with joint forces

### Canopy States

| Phase | Trigger | Drag Area | Visual | Body Damping |
|-------|---------|-----------|--------|--------------|
| Inflating | Deploy event | Full | Depth 0→target (0.8s ease-out) | Default |
| Open | Inflation complete | Full | Flutter ±8% at 4Hz | Default |
| Collapsed | Rocket grounded | 5% of full | Depth→0 | LinearDamping=4, AngularDamping=4 |

## Mass Model

Composite mass from four parts, combined via parallel axis theorem (`rocket.rs`):

| Part | Shape | Density Source | CoM Location |
|------|-------|---------------|-------------|
| Body | Hollow cylinder | `body_density_kg_m3` × wall thickness | Geometric center |
| Nose cone | Conical shell | `nose_density_kg_m3` × wall thickness | 1/3 up from base |
| Motor | Point mass (0.04 kg) | Fixed | Near body base |
| Fins | Triangular plates | `fin_density_kg_m3` | 1/3 from root edge |

### Material Presets

| Material | Body ρ (kg/m³) | Wall (mm) | Typical Total Mass |
|----------|---------------|-----------|-------------------|
| Light | 400 | 0.8 | ~80g |
| Medium (default) | 700 | 1.2 | ~140g |
| Heavy | 1200 | 2.0 | ~300g |
| Very Heavy | 2700 | 0.5 | ~180g |

Motor mass (0.04 kg) is constant across all presets.

## Drag Models

### Rocket Body Aerodynamic Drag (`drag.rs`)

Proper v² model with axial/lateral decomposition:

```
F_axial  = -0.5 × Cd_axial × (π r²) × ρ × v_axial²
F_lateral = -0.5 × 1.2 × (length × 2r) × ρ × v_lateral²
```

- `Cd_axial` derived from cone half-angle: 0.15 (slender) → 0.50 (blunt)
- Lateral Cd hardcoded at 1.2
- Total capped at 20 N
- Active during `Launched` and `Descending` states
- Accounts for wind (uses relative airspeed)

### Detached Cone Drag (`drag.rs`)

Same v² model applied to the separated nosecone:

- Axial Cd from cone geometry
- Lateral Cd = 1.2, lateral area = `cone_length × 2r`
- Applied at cone base offset for aerodynamic torque
- Capped at 20 N

### Parachute Drag (`parachute.rs`)

```
F = -0.5 × 0.8 × (π r²) × 1.225 × v²     (capped at 50 N)
```

- Cd = 0.8 (hemispherical canopy)
- Area = full canopy circle (or 5% when collapsed)
- Applied to ParachuteBody; transmits to rocket via joints
- Uses relative airspeed (accounts for wind)

### Terminal Velocity Reference

For default rocket (~142g) at various parachute diameters:

| Diameter | Area (m²) | Terminal V (m/s) | Notes |
|----------|-----------|-------------------|-------|
| 0.3 | 0.071 | 6.3 | Too fast |
| 0.4 | 0.126 | 4.8 | NAR recommended range |
| 0.5 | 0.196 | 3.8 | Gentle drift |
| 0.6 | 0.283 | 3.2 | Very slow |

## Wind Model (`wind.rs`)

Procedural multi-frequency oscillation:

- Horizontal: 3 frequencies (periods 48s, 17s, 7s), max 8 m/s
- Vertical: 2 frequencies (periods 23s, 11s), max 2.5 m/s
- Force applied at variable center-of-pressure point along body axis
- CP offset retargets every 0.5–1.4s with smooth interpolation

**Note:** Despite appearances, `axial * axial.length()` IS v²-scaled (`v_dir × |v|²`). The coefficients (0.0005, 0.0012) are opaque lumped constants rather than explicit `0.5 × Cd × A × ρ`, but velocity dependence is physically correct. Force capped at 0.05 N.

## Force Application Schedule

All force systems run in `FixedPostUpdate` / `PhysicsSystems::First` (before avian3d's solver step):

| System | Source | Target |
|--------|--------|--------|
| `update_forces_system` | ForceTimer (thrust) | Rocket |
| `apply_wind_force_system` | Wind at variable CP | Rocket |
| `apply_aerodynamic_drag_system` | v² body drag | Rocket |
| `apply_cone_drag_system` | v² cone drag | DetachedCone |
| `parachute_drag_system` | v² chute drag | ParachuteBody |

Visual tracking systems (camera, shock cord, shroud lines) run in `PostUpdate` after `PhysicsSystems::Writeback`.

## Collision & Contact

| Entity | Layer | Collides With | Friction | Restitution |
|--------|-------|---------------|----------|-------------|
| Rocket body | Rocket | Ground | 0.7 | 0.4 |
| Rocket cone | Rocket | Ground | 0.7 | 0.4 |
| Detached cone | Debris | Ground | 0.5 | 0.3 |
| Parachute body | Debris | Ground | 0.8 | 0.1 |
| Ground | Ground | All | 0.7 | 0.2 |

Landing detected when rocket touches ground AND `velocity.y ≤ 0.25 m/s`.

## Known Limitations

1. **No geometry-based aerodynamic stability** — CP offset is randomized, not derived from fin size/position; fins affect mass but not aerodynamic behavior. A finned rocket should weathercock; an unfinned one should tumble.
2. **No crash detection** — high-speed impacts bounce (restitution 0.4) until velocity drops below 0.25 m/s threshold. Crashes should feel terminal.
3. **Cone inertia approximation** — uses cylinder formula instead of proper conical shell (minor impact on tumble dynamics)
4. **Constant air density** — hardcoded at sea level (1.225 kg/m³); negligible at model rocket altitudes
5. **Wind coefficients are opaque** — lumped constants (0.0005, 0.0012) rather than derived from geometry; v² scaling is correct
