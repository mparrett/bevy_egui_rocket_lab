use bevy::prelude::*;
use bevy_math::primitives::Cylinder;
use bevy_xpbd_3d::plugins::collision::Collider;
use bevy_xpbd_3d::prelude::*;

use crate::cone::Cone;
use crate::fin::Fin;
use crate::physics::TimedForces;

#[derive(Component)]
pub struct RocketMarker;

#[derive(Component, Default)]
pub struct RocketBody;

#[derive(Component, Default)]
pub struct RocketCone;

#[derive(Component)]
pub struct FinMarker;

const CONE_DENISTY: f32 = 1.0;
const FUSELAGE_DENSITY: f32 = 1.0;
//const FIN_DENSITY: f32 = 0.025;

/*
// TODO: Refactor to use ECS properly
#[derive(Component)]
struct Length(f32);

#[derive(Component)]
struct Height(f32);


#[derive(Component)]
struct Radius(f32);


#[derive(Component)]
struct NumFins(f32);


#[derive(Bundle)]
struct Size2D {
    length: Length,
    height: Height,
}

#[derive(Bundle)]
struct RocketBundle {
  size: Size2D,
  radius: Radius,
  num_fins: NumFins,
  fin_size: Size2D
}
*/

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
    pub is_ignited: bool,
    pub is_grounded: bool,
    pub max_height: f32,
    pub max_velocity: f32,
    pub additional_mass: f32,
    pub engine_angle: f32,
    pub state: RocketStateEnum,
}
impl Default for RocketState {
    fn default() -> Self {
        RocketState {
            is_grounded: false,
            is_ignited: false,
            max_height: 0.0,
            max_velocity: 0.0,
            additional_mass: 1.0,
            engine_angle: 0.0,
            state: RocketStateEnum::Grounded,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    rocket_dims: &RocketDimensions,
    meshes: &mut Assets<Mesh>,
    rocket_color_hex: &str,
) -> Vec<PbrBundle> {
    let n_fins = rocket_dims.num_fins as usize;
    let degs_per_fin = 360.0 / n_fins as f32;
    // Position of the central object (e.g., the rocket body)
    let central_position = Vec3::new(0.0, -rocket_dims.total_length() * 0.5, 0.0);
    let distance_from_center = rocket_dims.radius; // Distance from the center to the base of each fin

    let fin_mesh = Mesh::from(Fin {
        height: rocket_dims.fin_height,
        length: rocket_dims.fin_length,
        width: 0.015,
    });

    /*
    let fin_material = StandardMaterial {
        base_color: Color::hex(rocket_color_hex).unwrap(),
        ..Default::default()
    };
    */
    let fin_material = StandardMaterial {
        base_color: Color::hex(rocket_color_hex).unwrap(),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        reflectance: 0.6,
        ..default()
    };

    // Create and position fins
    let mut bundles: Vec<PbrBundle> = Vec::new();
    for i in 0..n_fins {
        let fin_mat_handle = materials.add(fin_material.clone());
        let angle = i as f32 * degs_per_fin.to_radians(); // x degrees between each fin for 3 fins
        let rotation = Quat::from_rotation_y(angle);

        // Calculate the position of each fin's base relative to the central object
        let position_relative =
            central_position + rotation * Vec3::new(0.0, 0.0, distance_from_center);

        // Combine the rotation with an adjustment so the fin points outwards
        let fin_rotation = rotation * Quat::from_rotation_y(-90.0f32.to_radians());

        bundles.push(PbrBundle {
            mesh: meshes.add(fin_mesh.clone()),
            material: fin_mat_handle,
            transform: Transform {
                translation: position_relative,
                rotation: fin_rotation,
                ..Default::default()
            },
            ..Default::default()
        });
    }
    bundles
}

pub fn locked_axes() {
    LockedAxes::new()
        .lock_rotation_x()
        .lock_rotation_y()
        .lock_rotation_z()
        .lock_translation_x()
        .lock_translation_y()
        .lock_translation_z();
}

pub fn spawn_rocket_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rocket_dims: Res<RocketDimensions>,
    _rocket_state: ResMut<RocketState>,
) {
    // Make sure to set the attached colliders' densities to zero if you want your
    // explicit values to be the final mass-properties values.

    let rocket_color_hex = "#eeeeff";
    let rocket_material = StandardMaterial {
        base_color: Color::hex(rocket_color_hex).unwrap(),
        metallic: 0.4,
        perceptual_roughness: 0.4,
        reflectance: 0.6,
        emissive: Color::WHITE,
        ..default()
    };

    // TODO: Verify local forces being calculated correctly
    // https://docs.rs/bevy_xpbd_3d/0.4.2/bevy_xpbd_3d/components/struct.ExternalForce.html#local-forces

    let initial_rocket_pos = Transform::from_xyz(0.0, rocket_dims.total_length() * 0.5, 0.0);
    let rocket_bundle = (
        RigidBody::Dynamic,
        RocketMarker,
        //AdditionalMassProperties::Mass(rocket_state.additional_mass),
        ExternalForce::ZERO.with_persistence(false),
        ExternalImpulse::ZERO.with_persistence(false),
        ExternalTorque::ZERO.with_persistence(false),
        AngularVelocity::ZERO,
        LinearVelocity::ZERO,
        TimedForces::default(),
        LinearDamping(0.4), // Simulate air resistance. TODO: Dynamic based on rocket shape?
        AngularDamping(1.0), // Simulate air resistance. TODO: Dynamic based on rocket shape?
        SpatialBundle::from_transform(initial_rocket_pos),
        locked_axes(),
        Name::new("Rocket"),
    );

    commands.spawn(rocket_bundle).with_children(|parent| {
        parent.spawn((
            PbrBundle {
                mesh: meshes.add(Cylinder::new(rocket_dims.radius, rocket_dims.length).mesh()),
                //material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: materials.add(rocket_material.clone()),
                ..Default::default()
            },
            Collider::cylinder(rocket_dims.length, rocket_dims.radius),
            RocketBody,
            ColliderDensity(FUSELAGE_DENSITY),
            Friction::new(0.7),
            Restitution::new(0.4),
            Name::new("RocketBody"),
        ));
        parent.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cone {
                    radius: rocket_dims.radius,
                    height: rocket_dims.cone_length,
                    segments: 8,
                })),
                //material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
                material: materials.add(rocket_material),
                transform: Transform::from_xyz(0.0, rocket_dims.total_length() * 0.5, 0.0),
                ..Default::default()
            },
            Collider::cone(rocket_dims.cone_length, rocket_dims.radius),
            ColliderDensity(CONE_DENISTY),
            Friction::new(0.7),
            Restitution::new(0.4),
            RocketCone,
            Name::new("RocketCone"),
        ));

        let rocket_fin_pbr_bundles = create_rocket_fin_pbr_bundles(
            materials,
            rocket_dims.as_ref(),
            meshes.as_mut(),
            rocket_color_hex,
        );
        for bundle in rocket_fin_pbr_bundles {
            parent.spawn((bundle, FinMarker)); // Spawn the fin entities
        }
    });
}
