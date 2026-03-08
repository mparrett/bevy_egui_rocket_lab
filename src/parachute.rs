use avian3d::prelude::*;
use bevy::{math::primitives::Cylinder, prelude::*};

use crate::rocket::{
    RocketCone, RocketDimensions, RocketMarker, RocketState, RocketStateEnum,
};
use crate::ResetEvent;

#[derive(Component)]
pub struct DetachedCone;

#[derive(Component)]
pub struct ShockCord;

#[derive(Component)]
pub struct ParachuteVisual;

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

    let tube_top_offset = Vec3::Y * rocket_dims.length * 0.5;
    let chute_mesh = meshes.add(Sphere::new(0.05).mesh().ico(2).unwrap());
    let chute_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.0),
        emissive: LinearRgba::new(1.0, 0.3, 0.0, 1.0),
        ..default()
    });
    commands.entity(rocket_ent).with_children(|parent| {
        parent.spawn((
            Mesh3d(chute_mesh),
            MeshMaterial3d(chute_material),
            Transform::from_translation(tube_top_offset + Vec3::Y * 0.08),
            ParachuteVisual,
            Name::new("ParachuteVisual"),
        ));
    });

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
) {
    if reset_events.read().next().is_none() {
        return;
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

    parachute_config.deployed = false;
    rocket_dims.flag_changed = true;
}
