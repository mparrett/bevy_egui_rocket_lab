use avian3d::prelude::*;
use bevy::{math::primitives::Cylinder, mesh::VertexAttributeValues, prelude::*};

use crate::canopy::SphericalCap;
use crate::rocket::{
    RocketCone, RocketDimensions, RocketMarker, RocketState, RocketStateEnum,
};
use crate::ResetEvent;

pub const EJECTION_DELAY_SECS: f32 = 3.0;

const RADIAL_SEGMENTS: u32 = 12;
const RING_COUNT: u32 = 4;
const SHROUD_LINE_COUNT: u32 = 6;
const CAP_DEPTH_RATIO: f32 = 0.4;
const INFLATION_SECS: f32 = 0.8;
const FLUTTER_AMPLITUDE: f32 = 0.08;
const FLUTTER_FREQ: f32 = 4.0;
const MAX_TILT_RAD: f32 = 15.0 * std::f32::consts::PI / 180.0;
const SHROUD_CORD_RADIUS: f32 = 0.001;

#[derive(Component)]
pub struct EjectionTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct DetachedCone;

#[derive(Component)]
pub struct ShockCord;

#[derive(Component)]
pub struct ParachuteVisual;

#[derive(Component)]
pub struct ShroudLine {
    pub rim_index: u32,
}

pub enum CanopyPhase {
    Inflating,
    Open,
}

#[derive(Component)]
pub struct CanopyAnimation {
    pub timer: Timer,
    pub phase: CanopyPhase,
    pub flutter_time: f32,
    pub target_depth: f32,
    pub current_depth: f32,
    pub rim_radius: f32,
}

#[derive(Resource)]
pub struct ParachuteConfig {
    pub diameter: f32,
    pub deployed: bool,
}

impl Default for ParachuteConfig {
    fn default() -> Self {
        Self {
            diameter: 0.3,
            deployed: false,
        }
    }
}

#[derive(Message)]
pub struct DeployParachuteEvent;

pub fn auto_deploy_parachute_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut EjectionTimer), With<RocketMarker>>,
    mut deploy_writer: MessageWriter<DeployParachuteEvent>,
    mut commands: Commands,
) {
    let Ok((entity, mut ejection)) = query.single_mut() else {
        return;
    };
    ejection.timer.tick(time.delta());
    if ejection.timer.just_finished() {
        deploy_writer.write(DeployParachuteEvent);
        commands.entity(entity).remove::<EjectionTimer>();
    }
}

pub fn deploy_parachute_system(
    mut commands: Commands,
    mut deploy_events: MessageReader<DeployParachuteEvent>,
    mut rocket_state: ResMut<RocketState>,
    mut parachute_config: ResMut<ParachuteConfig>,
    rocket_dims: Res<RocketDimensions>,
    rocket_query: Query<(Entity, &Transform), With<RocketMarker>>,
    cone_query: Query<(Entity, &GlobalTransform), With<RocketCone>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if deploy_events.read().next().is_none() {
        return;
    }
    if rocket_state.state != RocketStateEnum::Launched {
        return;
    }
    if parachute_config.deployed {
        return;
    }

    let Ok((rocket_ent, rocket_transform)) = rocket_query.single() else {
        return;
    };
    let Ok((cone_ent, cone_global)) = cone_query.single() else {
        return;
    };

    rocket_state.state = RocketStateEnum::Descending;
    parachute_config.deployed = true;

    // Detach cone
    let cone_world_pos = cone_global.translation();
    let cone_world_rot = cone_global.to_isometry().rotation;
    commands.entity(cone_ent).remove_parent_in_place();
    commands.entity(cone_ent).insert((
        DetachedCone,
        Transform::from_translation(cone_world_pos).with_rotation(cone_world_rot),
        RigidBody::Dynamic,
        LinearDamping(2.0),
        LinearVelocity(Vec3::new(0.2, 1.5, 0.1)),
    ));
    commands.entity(cone_ent).remove::<RocketCone>();

    // Shock cord
    let cord_mesh = meshes.add(
        Cylinder::new(0.002, 1.0)
            .mesh()
            .resolution(6),
    );
    let cord_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.53, 0.3),
        ..default()
    });
    commands.spawn((
        Mesh3d(cord_mesh),
        MeshMaterial3d(cord_material),
        Transform::default(),
        ShockCord,
        Visibility::default(),
        Name::new("ShockCord"),
    ));

    // Canopy (spherical cap)
    let rim_radius = parachute_config.diameter * 0.5;
    let target_depth = rim_radius * CAP_DEPTH_RATIO;
    let initial_depth = 0.001;

    let cap = SphericalCap {
        radius: rim_radius,
        depth: initial_depth,
        radial_segments: RADIAL_SEGMENTS,
        ring_count: RING_COUNT,
    };
    let chute_mesh = meshes.add(Mesh::from(cap));
    let chute_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.0),
        emissive: LinearRgba::new(1.0, 0.3, 0.0, 1.0),
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let tube_top_offset = Vec3::Y * rocket_dims.length * 0.5;
    commands.entity(rocket_ent).with_children(|parent| {
        parent.spawn((
            Mesh3d(chute_mesh),
            MeshMaterial3d(chute_material),
            Transform::from_translation(tube_top_offset + Vec3::Y * 0.08),
            ParachuteVisual,
            CanopyAnimation {
                timer: Timer::from_seconds(INFLATION_SECS, TimerMode::Once),
                phase: CanopyPhase::Inflating,
                flutter_time: 0.0,
                target_depth,
                current_depth: initial_depth,
                rim_radius,
            },
            Name::new("ParachuteVisual"),
        ));
    });

    // Shroud lines (every other radial segment → 6 lines from 12 segments)
    let line_mesh = meshes.add(
        Cylinder::new(SHROUD_CORD_RADIUS, 1.0)
            .mesh()
            .resolution(4),
    );
    let line_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        ..default()
    });
    for i in 0..SHROUD_LINE_COUNT {
        let rim_index = i * 2; // every other segment
        commands.spawn((
            Mesh3d(line_mesh.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::default(),
            ShroudLine { rim_index },
            Visibility::default(),
            Name::new(format!("ShroudLine_{}", i)),
        ));
    }

    info!(
        "Parachute deployed at altitude {:.1}m",
        rocket_transform.translation.y
    );
}

pub fn parachute_drag_system(
    rocket_state: Res<RocketState>,
    parachute_config: Res<ParachuteConfig>,
    mut query: Query<Forces, With<RocketMarker>>,
) {
    if rocket_state.state != RocketStateEnum::Descending {
        return;
    }
    let Ok(mut forces) = query.single_mut() else {
        return;
    };

    let velocity = forces.linear_velocity();
    let speed_sq = velocity.length_squared();
    if speed_sq < 1e-6 {
        return;
    }

    let cd = 0.8;
    let rho = 1.225;
    let r = parachute_config.diameter * 0.5;
    let area = std::f32::consts::PI * r * r;

    let speed = speed_sq.sqrt();
    let drag_magnitude = (0.5 * cd * area * rho * speed_sq).min(50.0);
    let drag_force = -velocity / speed * drag_magnitude;
    forces.apply_force(drag_force);
}

pub fn animate_canopy_system(
    time: Res<Time>,
    rocket_state: Res<RocketState>,
    mut canopy_query: Query<(&mut CanopyAnimation, &mut Transform, &Mesh3d), With<ParachuteVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    rocket_query: Query<&LinearVelocity, With<RocketMarker>>,
) {
    if rocket_state.state != RocketStateEnum::Descending {
        return;
    }
    let Ok((mut anim, mut canopy_tf, mesh3d)) = canopy_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    anim.flutter_time += dt;

    match anim.phase {
        CanopyPhase::Inflating => {
            anim.timer.tick(time.delta());
            let t = anim.timer.fraction();
            // Ease-out: 1 - (1-t)^2
            let eased = 1.0 - (1.0 - t) * (1.0 - t);
            anim.current_depth = 0.001_f32.lerp(anim.target_depth, eased);
            if anim.timer.is_finished() {
                anim.phase = CanopyPhase::Open;
                anim.current_depth = anim.target_depth;
            }
        }
        CanopyPhase::Open => {
            let flutter = (anim.flutter_time * FLUTTER_FREQ).sin() * FLUTTER_AMPLITUDE * anim.target_depth;
            anim.current_depth = anim.target_depth + flutter;
        }
    }

    // Update mesh in-place
    if let Some(mesh) = meshes.get_mut(&mesh3d.0) {
        let new_mesh = Mesh::from(SphericalCap {
            radius: anim.rim_radius,
            depth: anim.current_depth.max(0.001),
            radial_segments: RADIAL_SEGMENTS,
            ring_count: RING_COUNT,
        });
        if let Some(VertexAttributeValues::Float32x3(new_positions)) =
            new_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions.clone());
        }
        if let Some(VertexAttributeValues::Float32x3(new_normals)) =
            new_mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals.clone());
        }
    }

    // Velocity-lag tilt
    if let Ok(vel) = rocket_query.single() {
        let horizontal = Vec3::new(vel.x, 0.0, vel.z);
        let h_speed = horizontal.length();
        if h_speed > 0.1 {
            let tilt_amount = (h_speed * 0.1).min(MAX_TILT_RAD);
            let tilt_dir = horizontal.normalize();
            let tilt_axis = Vec3::Y.cross(tilt_dir).normalize();
            if tilt_axis.length_squared() > 0.5 {
                canopy_tf.rotation = Quat::from_axis_angle(tilt_axis, tilt_amount);
            }
        } else {
            canopy_tf.rotation = Quat::IDENTITY;
        }
    }
}

pub fn update_shroud_lines_system(
    rocket_state: Res<RocketState>,
    rocket_query: Query<&Transform, With<RocketMarker>>,
    rocket_dims: Res<RocketDimensions>,
    canopy_query: Query<(&GlobalTransform, &CanopyAnimation), With<ParachuteVisual>>,
    mut line_query: Query<
        (&ShroudLine, &mut Transform),
        (Without<RocketMarker>, Without<ParachuteVisual>),
    >,
) {
    if rocket_state.state != RocketStateEnum::Descending {
        return;
    }
    let Ok(rocket_tf) = rocket_query.single() else {
        return;
    };
    let Ok((canopy_global, anim)) = canopy_query.single() else {
        return;
    };

    let tube_top = rocket_tf.translation
        + rocket_tf.rotation * (Vec3::Y * rocket_dims.length * 0.5);

    let tau = std::f32::consts::TAU;
    let canopy_rot = canopy_global.to_isometry().rotation;
    let canopy_pos = canopy_global.translation();

    for (shroud, mut line_tf) in &mut line_query {
        let phi = shroud.rim_index as f32 / RADIAL_SEGMENTS as f32 * tau;
        // Rim vertex in canopy local space (rim is at y=0 in the cap mesh)
        let local_rim = Vec3::new(
            anim.rim_radius * phi.cos(),
            0.0,
            anim.rim_radius * phi.sin(),
        );
        let rim_world = canopy_pos + canopy_rot * local_rim;

        let midpoint = (tube_top + rim_world) * 0.5;
        let diff = rim_world - tube_top;
        let distance = diff.length();

        line_tf.translation = midpoint;
        line_tf.scale = Vec3::new(1.0, distance, 1.0);
        if distance > 1e-4 {
            let dir = diff / distance;
            line_tf.rotation = Quat::from_rotation_arc(Vec3::Y, dir);
        }
    }
}

pub fn update_shock_cord_system(
    rocket_state: Res<RocketState>,
    rocket_query: Query<&Transform, With<RocketMarker>>,
    cone_query: Query<&Transform, With<DetachedCone>>,
    rocket_dims: Res<RocketDimensions>,
    mut cord_query: Query<&mut Transform, (With<ShockCord>, Without<RocketMarker>, Without<DetachedCone>)>,
) {
    if rocket_state.state != RocketStateEnum::Descending {
        return;
    }
    let Ok(rocket_tf) = rocket_query.single() else {
        return;
    };
    let Ok(cone_tf) = cone_query.single() else {
        return;
    };
    let Ok(mut cord_tf) = cord_query.single_mut() else {
        return;
    };

    let tube_top = rocket_tf.translation + rocket_tf.rotation * (Vec3::Y * rocket_dims.length * 0.5);
    let cone_base = cone_tf.translation;
    let midpoint = (tube_top + cone_base) * 0.5;
    let diff = cone_base - tube_top;
    let distance = diff.length();

    cord_tf.translation = midpoint;
    cord_tf.scale = Vec3::new(1.0, distance, 1.0);
    if distance > 1e-4 {
        let dir = diff / distance;
        cord_tf.rotation = Quat::from_rotation_arc(Vec3::Y, dir);
    }
}

pub fn cleanup_parachute_system(
    mut commands: Commands,
    mut reset_events: MessageReader<ResetEvent>,
    mut parachute_config: ResMut<ParachuteConfig>,
    mut rocket_dims: ResMut<RocketDimensions>,
    cone_query: Query<Entity, With<DetachedCone>>,
    cord_query: Query<Entity, With<ShockCord>>,
    chute_query: Query<Entity, With<ParachuteVisual>>,
    shroud_query: Query<Entity, With<ShroudLine>>,
    rocket_query: Query<Entity, (With<RocketMarker>, With<EjectionTimer>)>,
) {
    if reset_events.read().next().is_none() {
        return;
    }

    for entity in &rocket_query {
        commands.entity(entity).remove::<EjectionTimer>();
    }

    if !parachute_config.deployed {
        return;
    }

    for entity in &cone_query {
        commands.entity(entity).despawn();
    }
    for entity in &cord_query {
        commands.entity(entity).despawn();
    }
    for entity in &chute_query {
        commands.entity(entity).despawn();
    }
    for entity in &shroud_query {
        commands.entity(entity).despawn();
    }

    parachute_config.deployed = false;
    rocket_dims.flag_changed = true;
}
