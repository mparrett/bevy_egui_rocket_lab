use avian3d::prelude::*;
use bevy::{math::primitives::Cylinder, prelude::*};

use crate::cone::Cone;
use crate::fin::Fin;
use crate::physics::lock_all_axes;

#[derive(Component)]
pub struct RocketMarker;

#[derive(Component, Default)]
pub struct RocketBody;

#[derive(Component, Default)]
pub struct RocketCone;

#[derive(Component)]
pub struct FinMarker;

const CONE_DENSITY: f32 = 1.0;
const FUSELAGE_DENSITY: f32 = 1.0;

pub const CIRCLE_RESOLUTION: u32 = 16;

#[derive(Resource)]
pub struct RocketDimensions {
    pub radius: f32,
    pub length: f32,
    pub cone_length: f32,
    pub num_fins: f32,
    pub fin_height: f32,
    pub fin_length: f32,
    pub flag_changed: bool,
}

impl RocketDimensions {
    pub fn new(radius: f32, height: f32, cone_length: f32) -> Self {
        RocketDimensions {
            radius,
            length: height,
            cone_length,
            num_fins: 3.0,
            fin_height: 0.2,
            fin_length: 0.1,
            flag_changed: false,
        }
    }
    pub fn total_length(&self) -> f32 {
        self.length + self.cone_length
    }
}
impl Default for RocketDimensions {
    fn default() -> Self {
        RocketDimensions::new(0.025, 0.5, 0.1)
    }
}

#[derive(PartialEq)]
pub enum RocketStateEnum {
    Initial,
    Launched,
    Grounded,
}

#[derive(Resource)]
pub struct RocketState {
    pub max_height: f32,
    pub max_velocity: f32,
    pub launch_origin_y: f32,
    pub state: RocketStateEnum,
}
impl Default for RocketState {
    fn default() -> Self {
        RocketState {
            max_height: 0.0,
            max_velocity: 0.0,
            launch_origin_y: 0.0,
            state: RocketStateEnum::Initial,
        }
    }
}

#[derive(Resource)]
pub struct RocketFlightParameters {
    pub force: f32,
    pub duration: f32,
}
impl Default for RocketFlightParameters {
    fn default() -> Self {
        RocketFlightParameters {
            force: 0.2,
            duration: 1.0,
        }
    }
}

pub fn create_rocket_fin_pbr_bundles(
    materials: &mut Assets<StandardMaterial>,
    rocket_dims: &RocketDimensions,
    meshes: &mut Assets<Mesh>,
    rocket_color_hex: &str,
) -> Vec<(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform)> {
    let n_fins = rocket_dims.num_fins as usize;
    let degs_per_fin = 360.0 / n_fins as f32;
    let central_position = Vec3::new(0.0, -rocket_dims.total_length() * 0.5, 0.0);
    let distance_from_center = rocket_dims.radius;

    let fin_mesh = Mesh::from(Fin {
        height: rocket_dims.fin_height,
        length: rocket_dims.fin_length,
        width: 0.015,
    });

    let fin_material = StandardMaterial {
        base_color: Srgba::hex(rocket_color_hex)
            .expect("rocket_color_hex must be a valid hex literal")
            .into(),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        reflectance: 0.6,
        ..default()
    };

    let fin_mesh_handle = meshes.add(fin_mesh);
    let fin_mat_handle = materials.add(fin_material);

    let mut bundles = Vec::new();
    for i in 0..n_fins {
        let angle = i as f32 * degs_per_fin.to_radians();
        let rotation = Quat::from_rotation_y(angle);

        let position_relative =
            central_position + rotation * Vec3::new(0.0, 0.0, distance_from_center);

        let fin_rotation = rotation * Quat::from_rotation_y(-90.0f32.to_radians());

        bundles.push((
            Mesh3d(fin_mesh_handle.clone()),
            MeshMaterial3d(fin_mat_handle.clone()),
            Transform {
                translation: position_relative,
                rotation: fin_rotation,
                ..Default::default()
            },
        ));
    }
    bundles
}

pub fn spawn_rocket_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rocket_dims: Res<RocketDimensions>,
    _rocket_state: ResMut<RocketState>,
) {
    let rocket_color_hex = "#eeeeff";
    let rocket_material = StandardMaterial {
        base_color: Srgba::hex(rocket_color_hex)
            .expect("rocket_color_hex must be a valid hex literal")
            .into(),
        metallic: 0.4,
        perceptual_roughness: 0.4,
        reflectance: 0.6,
        emissive: LinearRgba::BLACK,
        ..default()
    };

    let initial_rocket_pos = Transform::from_xyz(0.0, rocket_dims.total_length() * 0.5, 0.0);
    let rocket_bundle = (
        RigidBody::Dynamic,
        TransformInterpolation,
        RocketMarker,
        AngularVelocity::ZERO,
        LinearVelocity::ZERO,
        LinearDamping(0.4),
        AngularDamping(1.0),
        initial_rocket_pos,
        Visibility::default(),
        lock_all_axes(LockedAxes::new()),
        Name::new("Rocket"),
    );

    commands.spawn(rocket_bundle).with_children(|parent| {
        parent.spawn((
            Mesh3d(
                meshes.add(
                    Cylinder::new(rocket_dims.radius, rocket_dims.length)
                        .mesh()
                        .resolution(CIRCLE_RESOLUTION),
                ),
            ),
            MeshMaterial3d(materials.add(rocket_material.clone())),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider::cylinder(rocket_dims.radius, rocket_dims.length),
            RocketBody,
            CollisionEventsEnabled,
            ColliderDensity(FUSELAGE_DENSITY),
            Friction::new(0.7),
            Restitution::new(0.4),
            Name::new("RocketBody"),
        ));
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cone {
                radius: rocket_dims.radius,
                height: rocket_dims.cone_length,
                segments: CIRCLE_RESOLUTION,
            }))),
            MeshMaterial3d(materials.add(rocket_material)),
            Transform::from_xyz(0.0, rocket_dims.total_length() * 0.5, 0.0),
            Collider::cone(rocket_dims.radius, rocket_dims.cone_length),
            ColliderDensity(CONE_DENSITY),
            Friction::new(0.7),
            Restitution::new(0.4),
            RocketCone,
            CollisionEventsEnabled,
            Name::new("RocketCone"),
        ));

        let rocket_fin_pbr_bundles = create_rocket_fin_pbr_bundles(
            materials.as_mut(),
            rocket_dims.as_ref(),
            meshes.as_mut(),
            rocket_color_hex,
        );
        for bundle in rocket_fin_pbr_bundles {
            parent.spawn((bundle, FinMarker));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fin_bundles_share_mesh_and_material_handles() {
        let mut materials = Assets::<StandardMaterial>::default();
        let mut meshes = Assets::<Mesh>::default();
        let dims = RocketDimensions {
            num_fins: 4.0,
            ..RocketDimensions::default()
        };

        let bundles = create_rocket_fin_pbr_bundles(&mut materials, &dims, &mut meshes, "#eeeeff");
        assert_eq!(bundles.len(), 4);

        let first_mesh = &bundles[0].0.0;
        let first_material = &bundles[0].1.0;
        for (mesh, material, _) in &bundles {
            assert_eq!(&mesh.0, first_mesh);
            assert_eq!(&material.0, first_material);
        }
    }
}
