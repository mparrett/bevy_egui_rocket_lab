use avian3d::prelude::*;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::physics::GameLayer;

const GROUND_SIZE: f32 = 1000.0;
const GROUND_TILE_REPEAT: f32 = 80.0;
const GROUND_HEIGHT: f32 = 0.01;

const GROUND_TEXTURE: &str = "textures/GroundGrassGreen002_COL_4K_1024_mip.ktx2";

use crate::rendering::update_mesh_uvs_for_number_of_tiles;
use crate::scene::OutdoorMarker;

#[derive(Component)]
pub struct TreeRoot;

pub fn setup_ground_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let ground_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(GROUND_TEXTURE)),
        base_color: Color::srgb(0.3, 0.5, 0.2),
        reflectance: 0.05,
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
        OutdoorMarker,
        RigidBody::Static,
        Mesh3d(ground_mesh_handle),
        MeshMaterial3d(ground_material_handle),
        Collider::cuboid(GROUND_SIZE, GROUND_HEIGHT, GROUND_SIZE),
        CollisionLayers::new([GameLayer::Ground], LayerMask::ALL),
        Friction::new(0.7),
        Restitution::new(0.2),
        Transform::from_xyz(0.0, GROUND_HEIGHT, 0.0),
    ));

    let tree_scene = asset_server.load("models/pine_trees.glb#Scene0");
    let tree_scale = 0.25;
    for (x, z, scale_mult, y_rot) in [
        (15.0_f32, -8.0_f32, 1.0_f32, 0.0_f32),
        (-12.0, -15.0, 0.9, 1.2),
        (20.0, 5.0, 1.1, 2.5),
        (-18.0, 10.0, 0.85, 4.0),
        (8.0, -20.0, 1.05, 0.8),
        (-25.0, -5.0, 0.95, 3.3),
    ] {
        let s = tree_scale * scale_mult;
        commands.spawn((
            OutdoorMarker,
            TreeRoot,
            SceneRoot(tree_scene.clone()),
            Transform {
                translation: Vec3::new(x, 0.0, z),
                rotation: Quat::from_rotation_y(y_rot),
                scale: Vec3::splat(s),
            },
        ));
    }
}

pub fn disable_tree_shadows(
    mut commands: Commands,
    trees: Query<&Children, With<TreeRoot>>,
    all_children: Query<&Children>,
    meshes: Query<Entity, (With<Mesh3d>, Without<NotShadowCaster>)>,
) {
    for children in &trees {
        let mut stack: Vec<Entity> = children.iter().collect();
        while let Some(entity) = stack.pop() {
            if meshes.contains(entity) {
                commands.entity(entity).insert(NotShadowCaster);
            }
            if let Ok(grandchildren) = all_children.get(entity) {
                stack.extend(grandchildren.iter());
            }
        }
    }
}
