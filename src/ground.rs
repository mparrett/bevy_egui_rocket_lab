use bevy::prelude::*;
use bevy_xpbd_3d::prelude::Collider;
use bevy_xpbd_3d::prelude::Friction;
use bevy_xpbd_3d::prelude::Restitution;
use bevy_xpbd_3d::prelude::RigidBody;

const GROUND_SIZE: f32 = 200.0;
const GROUND_TILE_REPEAT: f32 = 8.0;
const GROUND_HEIGHT: f32 = 0.01;

use crate::rendering::update_mesh_uvs_for_number_of_tiles;

pub fn setup_ground_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Use ktx2 for compression and mipmaps, otherwise looks pretty bad
    //let texture_handle = asset_server.load("textures/green+grass-1024x1024.ktx2");
    let texture_handle = asset_server.load("textures/GroundGrassGreen002_COL_4K_1024.png.ktx2");

    // this material renders a normal texture
    let ground_material_handle = materials.add(StandardMaterial {
        emissive: Color::rgb(0.7, 0.7, 0.7),
        reflectance: 0.01,
        emissive_texture: Some(texture_handle.clone()),
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Opaque,
        unlit: false,
        ..default()
    });

    // Simple Grass
    //let ground_material_handle = materials.add(Color::rgb(0.2, 0.8, 0.2));

    // Pbr Grass
    /*
    let ground_material_handle = materials.add(StandardMaterial {
        base_color: Color::hex("#229922").unwrap(),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });
     */

    //let mut ground_mesh: Mesh = Plane3d::default().mesh().size(GROUND_SIZE, GROUND_SIZE).into();
    let mut ground_mesh: Mesh = Plane3d::default()
        .mesh()
        .size(GROUND_SIZE, GROUND_SIZE)
        .into();

    update_mesh_uvs_for_number_of_tiles(&mut ground_mesh, (GROUND_TILE_REPEAT, GROUND_TILE_REPEAT));
    let ground_mesh_handle = meshes.add(ground_mesh);
    //let ground_mesh_handle: Handle<Mesh> = asset_server.load("uneven_ground_mesh.glb");

    // Create the ground.
    commands
        .spawn(RigidBody::Static)
        .insert(PbrBundle {
            mesh: ground_mesh_handle,
            material: ground_material_handle,
            ..Default::default()
        })
        .insert((
            Collider::cuboid(GROUND_SIZE, GROUND_HEIGHT, GROUND_SIZE),
            //ColliderDensity(0.1),
            Friction::new(0.7),
            Restitution::new(0.2),
            SpatialBundle::from(Transform::from_xyz(0.0, GROUND_HEIGHT, 0.0)),
        ));

    // Extra plane to cover the ground
    /*
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(GROUND_SIZE*4., GROUND_SIZE*4.)),
        material: materials.add(Color::rgb(0.2, 0.4, 0.1)),
        transform: Transform::from_xyz(0.0, -GROUND_HEIGHT - 0.01, 0.0),
        ..Default::default()
    });
     */
}
