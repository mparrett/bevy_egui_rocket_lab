use avian3d::prelude::*;
use bevy::prelude::*;

const GROUND_SIZE: f32 = 1000.0;
const GROUND_TILE_REPEAT: f32 = 80.0;
const GROUND_HEIGHT: f32 = 0.01;

const GROUND_TEXTURE: &str = "textures/GroundGrassGreen002_COL_4K_1024_mip.ktx2";

use crate::rendering::update_mesh_uvs_for_number_of_tiles;

pub fn setup_ground_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let ground_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(GROUND_TEXTURE)),
        base_color: Color::srgb(0.3, 0.5, 0.2),
        reflectance: 0.01,
        alpha_mode: AlphaMode::Opaque,
        unlit: false,
        ..default()
    });

    let mut ground_mesh: Mesh = Plane3d::default()
        .mesh()
        .size(GROUND_SIZE, GROUND_SIZE)
        .into();

    update_mesh_uvs_for_number_of_tiles(&mut ground_mesh, (GROUND_TILE_REPEAT, GROUND_TILE_REPEAT));
    let ground_mesh_handle = meshes.add(ground_mesh);

    commands.spawn((
        RigidBody::Static,
        Mesh3d(ground_mesh_handle),
        MeshMaterial3d(ground_material_handle),
        Collider::cuboid(GROUND_SIZE, GROUND_HEIGHT, GROUND_SIZE),
        Friction::new(0.7),
        Restitution::new(0.2),
        Transform::from_xyz(0.0, GROUND_HEIGHT, 0.0),
    ));
}
