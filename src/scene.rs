use avian3d::prelude::*;
use bevy::{math::primitives::Cylinder, prelude::*};

use crate::{
    camera::CameraProperties,
    physics::lock_all_axes,
    rocket::{RocketDimensions, RocketMarker, RocketState, RocketStateEnum},
    sky::{SkyProperties, SkyRenderMode},
    AppState,
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Lab), (spawn_lab_room, enter_lab))
            .add_systems(OnEnter(AppState::Launch), enter_launch)
            .add_systems(OnEnter(AppState::Store), (spawn_store_room, enter_store));
    }
}

#[derive(Component)]
pub struct OutdoorMarker;

#[derive(Component)]
struct IndoorRoom;

const TABLE_HEIGHT: f32 = 0.75;
const TABLE_WIDTH: f32 = 1.2;
const TABLE_DEPTH: f32 = 0.6;
const TABLE_THICKNESS: f32 = 0.05;
pub const TABLE_TOP_Y: f32 = TABLE_HEIGHT + TABLE_THICKNESS / 2.0;

const ROOM_WIDTH: f32 = 7.2; // X axis (window wall direction)
const ROOM_DEPTH: f32 = 6.0; // Z axis
const ROOM_HEIGHT: f32 = 3.0;
const WALL_THICKNESS: f32 = 0.1;

struct RoomConfig {
    despawn_state: AppState,
    wall_color: Color,
    has_posters: bool,
    has_window: bool,
    has_table: bool,
    ceiling_light_pos: Vec3,
}

fn spawn_room(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    config: &RoomConfig,
) {
    let wall_mat = materials.add(StandardMaterial {
        base_color: config.wall_color,
        perceptual_roughness: 0.9,
        ..default()
    });
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.42, 0.38),
        perceptual_roughness: 0.8,
        ..default()
    });
    let table_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.35, 0.2),
        perceptual_roughness: 0.6,
        ..default()
    });

    let despawn = DespawnOnExit(config.despawn_state);
    let half_x = ROOM_WIDTH / 2.0;
    let half_z = ROOM_DEPTH / 2.0;

    // Floor
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        RigidBody::Static,
        Collider::cuboid(ROOM_WIDTH, WALL_THICKNESS, ROOM_DEPTH),
        Mesh3d(meshes.add(Cuboid::new(ROOM_WIDTH, WALL_THICKNESS, ROOM_DEPTH))),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, -WALL_THICKNESS / 2.0, 0.0),
        Friction::new(0.7),
    ));

    // Outdoor ground plane (visible through door/window)
    let outdoor_ground_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.25, 0.12),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    });
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(outdoor_ground_mat),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));

    // Ceiling
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        Mesh3d(meshes.add(Cuboid::new(ROOM_WIDTH, WALL_THICKNESS, ROOM_DEPTH))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, ROOM_HEIGHT, 0.0),
    ));

    // Front wall (solid, z = +half_z)
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        Mesh3d(meshes.add(Cuboid::new(ROOM_WIDTH, ROOM_HEIGHT, WALL_THICKNESS))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, ROOM_HEIGHT / 2.0, half_z),
    ));

    // Right wall (solid, x = +half_x)
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        Mesh3d(meshes.add(Cuboid::new(WALL_THICKNESS, ROOM_HEIGHT, ROOM_DEPTH))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(half_x, ROOM_HEIGHT / 2.0, 0.0),
    ));

    // Posters on front wall (facing -Z into room)
    if config.has_posters {
        let poster_h = 1.2_f32;
        let poster_w = poster_h * (1024.0 / 1526.0);
        let poster_thickness = 0.005;
        let poster_mesh = meshes.add(Cuboid::new(poster_w, poster_h, poster_thickness));
        for (x_offset, texture_path) in [
            (-0.8, "textures/poster_earth.png"),
            (0.8, "textures/poster_grand_tour.png"),
        ] {
            let poster_mat = materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load(texture_path)),
                perceptual_roughness: 0.9,
                reflectance: 0.02,
                unlit: true,
                ..default()
            });
            commands.spawn((
                IndoorRoom,
                despawn.clone(),
                Mesh3d(poster_mesh.clone()),
                MeshMaterial3d(poster_mat),
                Transform::from_xyz(x_offset, 1.6, half_z - 0.06),
            ));
        }
    }

    // Left wall (x = -half_x) with doorway
    let door_w = 1.0_f32;
    let door_h = 2.4_f32;
    let door_center_z = 1.0_f32; // toward front, opposite side from window
    let wall_x = -half_x;

    // Section from z = -half_z to door left edge
    let sec_a_len = half_z + door_center_z - door_w / 2.0;
    if sec_a_len > 0.01 {
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(WALL_THICKNESS, ROOM_HEIGHT, sec_a_len))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(wall_x, ROOM_HEIGHT / 2.0, -half_z + sec_a_len / 2.0),
        ));
    }
    // Section from door right edge to z = +half_z
    let sec_b_len = half_z - door_center_z - door_w / 2.0;
    if sec_b_len > 0.01 {
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(WALL_THICKNESS, ROOM_HEIGHT, sec_b_len))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(wall_x, ROOM_HEIGHT / 2.0, half_z - sec_b_len / 2.0),
        ));
    }
    // Above door
    let above_door_h = ROOM_HEIGHT - door_h;
    if above_door_h > 0.01 {
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(WALL_THICKNESS, above_door_h, door_w))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(wall_x, door_h + above_door_h / 2.0, door_center_z),
        ));
    }

    // Back wall (z = -half_z)
    let wall_z = -half_z;
    if config.has_window {
        let win_w = 2.0_f32;
        let win_bottom = 1.0_f32;
        let win_top = 2.2_f32;

        // Left section
        let left_w = half_x - win_w / 2.0;
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(left_w, ROOM_HEIGHT, WALL_THICKNESS))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(-(half_x - left_w / 2.0), ROOM_HEIGHT / 2.0, wall_z),
        ));
        // Right section
        let right_w = half_x - win_w / 2.0;
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(right_w, ROOM_HEIGHT, WALL_THICKNESS))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(half_x - right_w / 2.0, ROOM_HEIGHT / 2.0, wall_z),
        ));
        // Below window
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(win_w, win_bottom, WALL_THICKNESS))),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(0.0, win_bottom / 2.0, wall_z),
        ));
        // Above window
        let above_h = ROOM_HEIGHT - win_top;
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(win_w, above_h, WALL_THICKNESS))),
            MeshMaterial3d(wall_mat),
            Transform::from_xyz(0.0, win_top + above_h / 2.0, wall_z),
        ));
    } else {
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(meshes.add(Cuboid::new(ROOM_WIDTH, ROOM_HEIGHT, WALL_THICKNESS))),
            MeshMaterial3d(wall_mat),
            Transform::from_xyz(0.0, ROOM_HEIGHT / 2.0, wall_z),
        ));
    }

    if config.has_table {
        // Table top
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            RigidBody::Static,
            Collider::cuboid(TABLE_WIDTH, TABLE_THICKNESS, TABLE_DEPTH),
            Mesh3d(meshes.add(Cuboid::new(TABLE_WIDTH, TABLE_THICKNESS, TABLE_DEPTH))),
            MeshMaterial3d(table_mat.clone()),
            Transform::from_xyz(0.0, TABLE_HEIGHT, 0.0),
            Friction::new(0.7),
        ));

        // Table legs (4 cylinders, IKEA style)
        let leg_radius = 0.025;
        let leg_height = TABLE_HEIGHT - TABLE_THICKNESS / 2.0;
        let leg_mesh = meshes.add(Cylinder::new(leg_radius, leg_height));
        let inset_x = TABLE_WIDTH / 2.0 - 0.06;
        let inset_z = TABLE_DEPTH / 2.0 - 0.06;
        for (x, z) in [
            (inset_x, inset_z),
            (-inset_x, inset_z),
            (inset_x, -inset_z),
            (-inset_x, -inset_z),
        ] {
            commands.spawn((
                IndoorRoom,
                despawn.clone(),
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(table_mat.clone()),
                Transform::from_xyz(x, leg_height / 2.0, z),
            ));
        }
    }

    // Ceiling light (softened)
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        PointLight {
            intensity: 240_000.0,
            range: 12.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            color: Color::srgb(1.0, 0.95, 0.88),
            ..default()
        },
        Transform::from_translation(config.ceiling_light_pos),
    ));
    // Fill light (no shadows, warm)
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        PointLight {
            intensity: 90_000.0,
            range: 10.0,
            shadows_enabled: false,
            color: Color::srgb(0.9, 0.92, 1.0),
            ..default()
        },
        Transform::from_xyz(-1.5, 1.5, -1.0),
    ));
    if config.has_window {
        // Window light (simulates outdoor light coming through)
        commands.spawn((
            IndoorRoom,
            despawn,
            PointLight {
                intensity: 180_000.0,
                range: 8.0,
                shadows_enabled: false,
                color: Color::srgb(0.85, 0.92, 1.0),
                ..default()
            },
            Transform::from_xyz(0.0, 1.6, -half_z + 0.3),
        ));
    }
}

fn spawn_lab_room(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    spawn_room(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &RoomConfig {
            despawn_state: AppState::Lab,
            wall_color: Color::srgb(0.7, 0.7, 0.72),
            has_posters: true,
            has_window: true,
            has_table: true,
            ceiling_light_pos: Vec3::new(0.5, ROOM_HEIGHT - 0.3, 0.3),
        },
    );
}

fn spawn_store_room(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    spawn_room(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &RoomConfig {
            despawn_state: AppState::Store,
            wall_color: Color::srgb(0.6, 0.55, 0.5),
            has_posters: false,
            has_window: false,
            has_table: false,
            ceiling_light_pos: Vec3::new(-0.5, ROOM_HEIGHT - 0.3, -0.2),
        },
    );

    let half_z = ROOM_DEPTH / 2.0;
    let despawn = DespawnOnExit(AppState::Store);

    // Counter (solid block with a top surface)
    let counter_w = 2.0;
    let counter_d = 0.5;
    let counter_h = TABLE_HEIGHT;
    let counter_top_thickness = 0.04;
    let counter_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.32, 0.18),
        perceptual_roughness: 0.65,
        ..default()
    });
    let base_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.25, 0.14),
        perceptual_roughness: 0.8,
        ..default()
    });
    let counter_z = -half_z + counter_d / 2.0 + WALL_THICKNESS + 0.3;
    // Top
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        RigidBody::Static,
        Collider::cuboid(counter_w, counter_top_thickness, counter_d),
        Mesh3d(meshes.add(Cuboid::new(counter_w, counter_top_thickness, counter_d))),
        MeshMaterial3d(counter_mat),
        Transform::from_xyz(0.0, counter_h, counter_z),
        Friction::new(0.7),
    ));
    // Solid base (slightly inset)
    let base_inset = 0.04;
    let base_h = counter_h - counter_top_thickness / 2.0;
    commands.spawn((
        IndoorRoom,
        despawn.clone(),
        Mesh3d(meshes.add(Cuboid::new(
            counter_w - base_inset * 2.0,
            base_h,
            counter_d - base_inset * 2.0,
        ))),
        MeshMaterial3d(base_mat),
        Transform::from_xyz(0.0, base_h / 2.0, counter_z),
    ));

    // Shelves on back wall
    let shelf_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.3, 0.18),
        perceptual_roughness: 0.7,
        ..default()
    });
    let shelf_w = 3.0;
    let shelf_d = 0.3;
    let shelf_h = 0.04;
    let shelf_mesh = meshes.add(Cuboid::new(shelf_w, shelf_h, shelf_d));
    let bottom_shelf_y = 1.0_f32;
    let shelf_z = -half_z + shelf_d / 2.0 + WALL_THICKNESS;
    for y in [bottom_shelf_y, 1.8] {
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(shelf_mesh.clone()),
            MeshMaterial3d(shelf_mat.clone()),
            Transform::from_xyz(0.0, y, shelf_z),
        ));
    }

    // Rocket motors on bottom shelf
    let motor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.32),
        perceptual_roughness: 0.4,
        metallic: 0.3,
        ..default()
    });
    let motor_radius = 0.013;
    let motor_length = 0.15;
    let motor_mesh = meshes.add(Cylinder::new(motor_radius, motor_length));
    for x in [-0.63, -0.58, -0.53, -0.48] {
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(motor_mesh.clone()),
            MeshMaterial3d(motor_mat.clone()),
            Transform::from_xyz(x, bottom_shelf_y + shelf_h / 2.0 + motor_radius, shelf_z),
        ));
    }

    // Overhead light fixtures (4 in a grid)
    let fixture_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        emissive: bevy::color::LinearRgba::new(2.0, 1.9, 1.7, 1.0),
        ..default()
    });
    let fixture_mesh = meshes.add(Cuboid::new(0.4, 0.03, 0.15));
    let light_y = ROOM_HEIGHT - 0.15;
    for (x, z) in [(-1.2, -1.0), (1.2, -1.0), (-1.2, 1.2), (1.2, 1.2)] {
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            PointLight {
                intensity: 120_000.0,
                range: 8.0,
                shadows_enabled: false,
                color: Color::srgb(1.0, 0.97, 0.92),
                ..default()
            },
            Transform::from_xyz(x, light_y, z),
        ));
        commands.spawn((
            IndoorRoom,
            despawn.clone(),
            Mesh3d(fixture_mesh.clone()),
            MeshMaterial3d(fixture_mat.clone()),
            Transform::from_xyz(x, ROOM_HEIGHT - 0.02, z),
        ));
    }
}

fn enter_indoor(
    outdoor_query: &mut Query<&mut Visibility, (With<OutdoorMarker>, Without<RocketMarker>)>,
    rocket_query: &mut Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut LockedAxes,
        ),
        With<RocketMarker>,
    >,
    rocket_state: &mut RocketState,
    rocket_dims: &RocketDimensions,
    camera_properties: &mut CameraProperties,
    camera_query: &mut Query<
        (Entity, &mut bevy::core_pipeline::tonemapping::Tonemapping),
        With<Camera3d>,
    >,
    commands: &mut Commands,
    show_rocket: bool,
) {
    for mut vis in outdoor_query.iter_mut() {
        *vis = Visibility::Hidden;
    }

    if let Ok((camera_entity, mut tonemapping)) = camera_query.single_mut() {
        // Remove atmosphere components and any Exposure carried over from atmosphere mode.
        // Reset tonemapper to the cubemap-mode default for consistent indoor rendering.
        commands.entity(camera_entity).remove::<(
            bevy::pbr::Atmosphere,
            bevy::pbr::AtmosphereSettings,
            bevy::light::AtmosphereEnvironmentMapLight,
            bevy::camera::Exposure,
        )>();
        *tonemapping = bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface;
    }

    rocket_state.state = RocketStateEnum::Initial;
    rocket_state.max_height = 0.0;
    rocket_state.max_velocity = 0.0;

    if let Ok((rocket_ent, mut transform, mut lin_vel, mut ang_vel, mut locked)) =
        rocket_query.single_mut()
    {
        let rocket_vis = if show_rocket {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        commands.entity(rocket_ent).insert(rocket_vis);
        let rocket_half = rocket_dims.length * 0.5;
        transform.translation = Vec3::new(0.0, TABLE_TOP_Y + rocket_half, 0.0);
        transform.rotation = Quat::IDENTITY;
        *lin_vel = LinearVelocity::ZERO;
        *ang_vel = AngularVelocity::ZERO;
        *locked = lock_all_axes(LockedAxes::new());
        rocket_state.launch_origin_y = transform.translation.y;
        camera_properties.target = transform.translation;
    }

    camera_properties.follow_mode = crate::camera::FollowMode::FreeLook;
    camera_properties.fixed_distance = 2.5;
    camera_properties.desired_translation = Vec3::new(-0.6, 1.5, 1.5);
    camera_properties.lagged_translation = camera_properties.desired_translation;
    camera_properties.lagged_translation_velocity = Vec3::ZERO;
    camera_properties.lagged_target = camera_properties.target;
    camera_properties.lagged_target_velocity = Vec3::ZERO;
}

fn enter_lab(
    mut outdoor_query: Query<&mut Visibility, (With<OutdoorMarker>, Without<RocketMarker>)>,
    mut rocket_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut LockedAxes,
        ),
        With<RocketMarker>,
    >,
    mut rocket_state: ResMut<RocketState>,
    rocket_dims: Res<RocketDimensions>,
    mut camera_properties: ResMut<CameraProperties>,
    mut camera_query: Query<
        (Entity, &mut bevy::core_pipeline::tonemapping::Tonemapping),
        With<Camera3d>,
    >,
    mut commands: Commands,
    mut sky_props: ResMut<SkyProperties>,
) {
    enter_indoor(
        &mut outdoor_query,
        &mut rocket_query,
        &mut rocket_state,
        &rocket_dims,
        &mut camera_properties,
        &mut camera_query,
        &mut commands,
        true,
    );
    sky_props.skybox_index = sky_props.lab_skybox_index;
    sky_props.skybox_changed = true;
}

fn enter_store(
    mut outdoor_query: Query<&mut Visibility, (With<OutdoorMarker>, Without<RocketMarker>)>,
    mut rocket_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut LockedAxes,
        ),
        With<RocketMarker>,
    >,
    mut rocket_state: ResMut<RocketState>,
    rocket_dims: Res<RocketDimensions>,
    mut camera_properties: ResMut<CameraProperties>,
    mut camera_query: Query<
        (Entity, &mut bevy::core_pipeline::tonemapping::Tonemapping),
        With<Camera3d>,
    >,
    mut commands: Commands,
    mut sky_props: ResMut<SkyProperties>,
) {
    enter_indoor(
        &mut outdoor_query,
        &mut rocket_query,
        &mut rocket_state,
        &rocket_dims,
        &mut camera_properties,
        &mut camera_query,
        &mut commands,
        false,
    );

    // Store camera: closer to counter/shelves on back wall
    camera_properties.target = Vec3::new(0.3, 1.15, -2.2);
    camera_properties.desired_translation = Vec3::new(-0.3, 1.3, -0.5);
    camera_properties.lagged_translation = camera_properties.desired_translation;
    camera_properties.lagged_target = camera_properties.target;
    camera_properties.fixed_distance = 1.5;

    sky_props.skybox_index = sky_props.store_skybox_index;
    sky_props.skybox_changed = true;
}

fn enter_launch(
    mut outdoor_query: Query<&mut Visibility, (With<OutdoorMarker>, Without<RocketMarker>)>,
    mut rocket_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut LockedAxes,
        ),
        With<RocketMarker>,
    >,
    mut rocket_state: ResMut<RocketState>,
    rocket_dims: Res<RocketDimensions>,
    mut camera_properties: ResMut<CameraProperties>,
    mut sky_mode: ResMut<SkyRenderMode>,
    mut sky_props: ResMut<SkyProperties>,
    mut commands: Commands,
) {
    for mut vis in &mut outdoor_query {
        *vis = Visibility::Visible;
    }

    // Force cubemap mode — bevy_firework's render pipeline is incompatible
    // with Atmosphere bind group bindings (29-31). See bevyengine/bevy#21784.
    *sky_mode = SkyRenderMode::Cubemap;
    sky_mode.set_changed();

    rocket_state.state = RocketStateEnum::Initial;
    rocket_state.max_height = 0.0;
    rocket_state.max_velocity = 0.0;

    if let Ok((rocket_ent, mut transform, mut lin_vel, mut ang_vel, mut locked)) =
        rocket_query.single_mut()
    {
        commands.entity(rocket_ent).insert(Visibility::Inherited);
        let rocket_half = rocket_dims.length * 0.5;
        transform.translation = Vec3::new(0.0, rocket_half, 0.0);
        transform.rotation = Quat::IDENTITY;
        *lin_vel = LinearVelocity::ZERO;
        *ang_vel = AngularVelocity::ZERO;
        *locked = lock_all_axes(LockedAxes::new());
        rocket_state.launch_origin_y = transform.translation.y;
        camera_properties.target = transform.translation;
    }

    *camera_properties = CameraProperties::default();
    if let Ok((_, transform, _, _, _)) = rocket_query.single() {
        camera_properties.target = transform.translation;
        camera_properties.lagged_target = transform.translation;
    }

    sky_props.skybox_index = sky_props.lab_skybox_index;
    sky_props.skybox_changed = true;
}
