use avian3d::prelude::*;
use bevy::{math::primitives::Cylinder, prelude::*};
use serde::{Deserialize, Serialize};

use crate::cone::Cone;
use crate::fin::Fin;
use crate::physics::{GameLayer, lock_all_axes};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorPreset {
    White,
    Red,
    Blue,
    Green,
    Yellow,
    Black,
}

impl ColorPreset {
    pub const ALL: [ColorPreset; 6] = [
        Self::White,
        Self::Red,
        Self::Blue,
        Self::Green,
        Self::Yellow,
        Self::Black,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::White => "White",
            Self::Red => "Red",
            Self::Blue => "Blue",
            Self::Green => "Green",
            Self::Yellow => "Yellow",
            Self::Black => "Black",
        }
    }

    pub fn to_color(self) -> Color {
        match self {
            Self::White => Color::srgb(0.93, 0.93, 1.0),
            Self::Red => Color::srgb(0.85, 0.15, 0.15),
            Self::Blue => Color::srgb(0.15, 0.3, 0.85),
            Self::Green => Color::srgb(0.2, 0.6, 0.2),
            Self::Yellow => Color::srgb(0.9, 0.8, 0.1),
            Self::Black => Color::srgb(0.08, 0.08, 0.08),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RocketMaterial {
    Light,
    Medium,
    Heavy,
    VeryHeavy,
}

impl RocketMaterial {
    pub const ALL: [RocketMaterial; 4] = [Self::Light, Self::Medium, Self::Heavy, Self::VeryHeavy];

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light (cardboard)",
            Self::Medium => "Medium (thick card)",
            Self::Heavy => "Heavy (plastic)",
            Self::VeryHeavy => "Very Heavy (metal)",
        }
    }

    pub fn price(self) -> f64 {
        match self {
            Self::Light => 0.0,
            Self::Medium => 5.0,
            Self::Heavy => 10.0,
            Self::VeryHeavy => 20.0,
        }
    }

    pub fn to_mass_model(self) -> RocketMassModel {
        match self {
            Self::Light => RocketMassModel {
                body_wall_thickness_m: 0.0008,
                nose_wall_thickness_m: 0.0006,
                fin_density_kg_m3: 300.0,
                body_density_kg_m3: 400.0,
                nose_density_kg_m3: 400.0,
                ..RocketMassModel::default()
            },
            Self::Medium => RocketMassModel::default(),
            Self::Heavy => RocketMassModel {
                body_wall_thickness_m: 0.0020,
                nose_wall_thickness_m: 0.0015,
                fin_density_kg_m3: 1000.0,
                body_density_kg_m3: 1200.0,
                nose_density_kg_m3: 1200.0,
                ..RocketMassModel::default()
            },
            Self::VeryHeavy => RocketMassModel {
                body_wall_thickness_m: 0.0005,
                nose_wall_thickness_m: 0.0004,
                fin_density_kg_m3: 2700.0,
                body_density_kg_m3: 2700.0,
                nose_density_kg_m3: 2700.0,
                ..RocketMassModel::default()
            },
        }
    }
}

#[derive(Component)]
pub struct RocketMarker;

#[derive(Component, Default)]
pub struct RocketBody;

#[derive(Component, Default)]
pub struct RocketCone;

#[derive(Component)]
pub struct FinMarker;

const FIN_THICKNESS_M: f32 = 0.0015;

pub const CIRCLE_RESOLUTION: u32 = 16;

#[derive(Resource, Clone)]
pub struct RocketMassModel {
    pub body_wall_thickness_m: f32,
    pub nose_wall_thickness_m: f32,
    pub fin_density_kg_m3: f32,
    pub body_density_kg_m3: f32,
    pub nose_density_kg_m3: f32,
    pub motor_mass_kg: f32,
    pub motor_length_m: f32,
}

impl Default for RocketMassModel {
    fn default() -> Self {
        Self {
            body_wall_thickness_m: 0.0012,
            nose_wall_thickness_m: 0.0009,
            fin_density_kg_m3: 500.0,
            body_density_kg_m3: 700.0,
            nose_density_kg_m3: 1050.0,
            motor_mass_kg: 0.040,
            motor_length_m: 0.070,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PartMassProperties {
    mass: f32,
    center_of_mass: Vec3,
    principal_inertia: Vec3,
}

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct RocketDimensions {
    pub radius: f32,
    pub length: f32,
    pub cone_length: f32,
    pub num_fins: f32,
    pub fin_height: f32,
    pub fin_length: f32,
    #[serde(default = "default_body_color")]
    pub body_color: ColorPreset,
    #[serde(default = "default_cone_color")]
    pub cone_color: ColorPreset,
    #[serde(default = "default_fin_color")]
    pub fin_color: ColorPreset,
    #[serde(default = "default_material")]
    pub material: RocketMaterial,
    #[serde(skip)]
    pub flag_changed: bool,
}

fn default_body_color() -> ColorPreset {
    ColorPreset::White
}
fn default_cone_color() -> ColorPreset {
    ColorPreset::White
}
fn default_fin_color() -> ColorPreset {
    ColorPreset::Green
}
fn default_material() -> RocketMaterial {
    RocketMaterial::Medium
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
            body_color: ColorPreset::White,
            cone_color: ColorPreset::White,
            fin_color: ColorPreset::Green,
            material: RocketMaterial::Medium,
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
    Descending,
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

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct RocketFlightParameters {
    pub force: f32,
    pub duration: f32,
}
impl Default for RocketFlightParameters {
    fn default() -> Self {
        RocketFlightParameters {
            force: 4.0,
            duration: 1.0,
        }
    }
}

fn cylinder_shell_part(
    radius: f32,
    length: f32,
    wall_thickness: f32,
    density: f32,
) -> PartMassProperties {
    let inner_radius = (radius - wall_thickness).max(0.0);
    let shell_volume =
        std::f32::consts::PI * (radius * radius - inner_radius * inner_radius) * length;
    let mass = shell_volume * density;
    let radial_term = radius * radius + inner_radius * inner_radius;
    let principal_inertia = Vec3::new(
        mass * (3.0 * radial_term + length * length) / 12.0,
        0.5 * mass * radial_term,
        mass * (3.0 * radial_term + length * length) / 12.0,
    );

    PartMassProperties {
        mass,
        center_of_mass: Vec3::ZERO,
        principal_inertia,
    }
}

fn cone_shell_part(
    radius: f32,
    body_length: f32,
    cone_length: f32,
    wall_thickness: f32,
    density: f32,
) -> PartMassProperties {
    let slant_height = (radius * radius + cone_length * cone_length).sqrt();
    let shell_volume = std::f32::consts::PI * radius * slant_height * wall_thickness;
    let mass = shell_volume * density;
    let center_of_mass = Vec3::new(0.0, body_length * 0.5 + cone_length / 3.0, 0.0);
    // Phase 1 approximation: use a matching-radius cylinder inertia for a stable explicit model.
    let principal_inertia = Vec3::new(
        mass * (3.0 * radius * radius + cone_length * cone_length) / 12.0,
        0.5 * mass * radius * radius,
        mass * (3.0 * radius * radius + cone_length * cone_length) / 12.0,
    );

    PartMassProperties {
        mass,
        center_of_mass,
        principal_inertia,
    }
}

fn motor_part(radius: f32, body_length: f32, model: &RocketMassModel) -> PartMassProperties {
    let motor_length = model.motor_length_m.min(body_length * 0.6).max(0.02);
    let motor_radius = (radius * 0.7).max(0.005);
    let mass = model.motor_mass_kg;
    let center_of_mass = Vec3::new(0.0, -body_length * 0.5 + motor_length * 0.5, 0.0);
    let principal_inertia = Vec3::new(
        mass * (3.0 * motor_radius * motor_radius + motor_length * motor_length) / 12.0,
        0.5 * mass * motor_radius * motor_radius,
        mass * (3.0 * motor_radius * motor_radius + motor_length * motor_length) / 12.0,
    );

    PartMassProperties {
        mass,
        center_of_mass,
        principal_inertia,
    }
}

fn fin_parts(rocket_dims: &RocketDimensions, model: &RocketMassModel) -> Vec<PartMassProperties> {
    let n_fins = rocket_dims.num_fins as usize;
    let degs_per_fin = 360.0 / n_fins as f32;
    let fin_volume = 0.5 * rocket_dims.fin_height * rocket_dims.fin_length * FIN_THICKNESS_M;
    let fin_mass = fin_volume * model.fin_density_kg_m3;
    let local_fin_com = Vec3::new(
        rocket_dims.fin_length / 3.0,
        rocket_dims.fin_height / 3.0,
        0.0,
    );
    let anchor = Vec3::new(0.0, -rocket_dims.length * 0.5, 0.0);

    (0..n_fins)
        .map(|i| {
            let angle = i as f32 * degs_per_fin.to_radians();
            let rotation = Quat::from_rotation_y(angle);
            let fin_rotation = rotation * Quat::from_rotation_y(-90.0f32.to_radians());
            let fin_translation = anchor + rotation * Vec3::new(0.0, 0.0, rocket_dims.radius);

            PartMassProperties {
                mass: fin_mass,
                center_of_mass: fin_translation + fin_rotation * local_fin_com,
                principal_inertia: Vec3::ZERO,
            }
        })
        .collect()
}

pub fn rocket_mass_properties(
    rocket_dims: &RocketDimensions,
    mass_model: &RocketMassModel,
) -> MassPropertiesBundle {
    let mut parts = Vec::new();
    parts.push(cylinder_shell_part(
        rocket_dims.radius,
        rocket_dims.length,
        mass_model.body_wall_thickness_m,
        mass_model.body_density_kg_m3,
    ));
    parts.push(cone_shell_part(
        rocket_dims.radius,
        rocket_dims.length,
        rocket_dims.cone_length,
        mass_model.nose_wall_thickness_m,
        mass_model.nose_density_kg_m3,
    ));
    parts.push(motor_part(
        rocket_dims.radius,
        rocket_dims.length,
        mass_model,
    ));
    parts.extend(fin_parts(rocket_dims, mass_model));

    let total_mass: f32 = parts.iter().map(|part| part.mass).sum();
    let center_of_mass = parts
        .iter()
        .map(|part| part.center_of_mass * part.mass)
        .sum::<Vec3>()
        / total_mass.max(1e-6);

    let principal_inertia = parts.iter().fold(Vec3::ZERO, |acc, part| {
        let offset = part.center_of_mass - center_of_mass;
        acc + part.principal_inertia
            + Vec3::new(
                part.mass * (offset.y * offset.y + offset.z * offset.z),
                part.mass * (offset.x * offset.x + offset.z * offset.z),
                part.mass * (offset.x * offset.x + offset.y * offset.y),
            )
    });

    MassPropertiesBundle {
        mass: Mass(total_mass.max(1e-6)),
        angular_inertia: AngularInertia::new(principal_inertia.max(Vec3::splat(1e-6))),
        center_of_mass: CenterOfMass(center_of_mass),
    }
}

pub fn create_rocket_fin_pbr_bundles(
    materials: &mut Assets<StandardMaterial>,
    rocket_dims: &RocketDimensions,
    meshes: &mut Assets<Mesh>,
    fin_color: Color,
) -> Vec<(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform)> {
    let n_fins = rocket_dims.num_fins as usize;
    let degs_per_fin = 360.0 / n_fins as f32;
    // Anchor fins to the base of the cylindrical body so nose/cone changes do
    // not shift body<->fin relative alignment.
    let central_position = Vec3::new(0.0, -rocket_dims.length * 0.5, 0.0);
    let distance_from_center = rocket_dims.radius;

    let fin_mesh = Mesh::from(Fin {
        height: rocket_dims.fin_height,
        length: rocket_dims.fin_length,
        width: FIN_THICKNESS_M,
    });

    let fin_material = StandardMaterial {
        base_color: fin_color,
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
    mass_model: Res<RocketMassModel>,
    _rocket_state: ResMut<RocketState>,
) {
    let body_material = StandardMaterial {
        base_color: rocket_dims.body_color.to_color(),
        metallic: 0.4,
        perceptual_roughness: 0.4,
        reflectance: 0.6,
        emissive: LinearRgba::BLACK,
        ..default()
    };
    let cone_material = StandardMaterial {
        base_color: rocket_dims.cone_color.to_color(),
        metallic: 0.4,
        perceptual_roughness: 0.4,
        reflectance: 0.6,
        emissive: LinearRgba::BLACK,
        ..default()
    };

    // Keep the rocket's body base at y=0 on spawn; cone extends upward from body top.
    let initial_rocket_pos = Transform::from_xyz(0.0, rocket_dims.length * 0.5, 0.0);
    let mass_properties = rocket_mass_properties(rocket_dims.as_ref(), mass_model.as_ref());
    let rocket_bundle = (
        RigidBody::Dynamic,
        TransformInterpolation,
        RocketMarker,
        mass_properties,
        NoAutoMass,
        NoAutoAngularInertia,
        NoAutoCenterOfMass,
        AngularVelocity::ZERO,
        LinearVelocity::ZERO,
        LinearDamping(0.0),
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
            MeshMaterial3d(materials.add(body_material)),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider::cylinder(rocket_dims.radius, rocket_dims.length),
            CollisionLayers::new([GameLayer::Rocket], [GameLayer::Ground]),
            RocketBody,
            CollisionEventsEnabled,
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
            MeshMaterial3d(materials.add(cone_material)),
            Transform::from_xyz(0.0, rocket_dims.total_length() * 0.5, 0.0),
            Collider::cone(rocket_dims.radius, rocket_dims.cone_length),
            CollisionLayers::new([GameLayer::Rocket], [GameLayer::Ground]),
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
            rocket_dims.fin_color.to_color(),
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

        let bundles =
            create_rocket_fin_pbr_bundles(&mut materials, &dims, &mut meshes, Color::WHITE);
        assert_eq!(bundles.len(), 4);

        let first_mesh = &bundles[0].0 .0;
        let first_material = &bundles[0].1 .0;
        for (mesh, material, _) in &bundles {
            assert_eq!(&mesh.0, first_mesh);
            assert_eq!(&material.0, first_material);
        }
    }

    #[test]
    fn fin_anchor_is_independent_of_cone_length() {
        let mut materials = Assets::<StandardMaterial>::default();
        let mut meshes = Assets::<Mesh>::default();
        let dims_short_cone = RocketDimensions {
            length: 1.0,
            cone_length: 0.1,
            num_fins: 3.0,
            ..RocketDimensions::default()
        };
        let dims_long_cone = RocketDimensions {
            length: 1.0,
            cone_length: 0.6,
            num_fins: 3.0,
            ..RocketDimensions::default()
        };

        let short = create_rocket_fin_pbr_bundles(
            &mut materials,
            &dims_short_cone,
            &mut meshes,
            Color::WHITE,
        );
        let long = create_rocket_fin_pbr_bundles(
            &mut materials,
            &dims_long_cone,
            &mut meshes,
            Color::WHITE,
        );

        assert_eq!(short.len(), long.len());
        for (short_bundle, long_bundle) in short.iter().zip(long.iter()) {
            assert_eq!(short_bundle.2.translation.y, long_bundle.2.translation.y);
            assert_eq!(short_bundle.2.translation.y, -dims_short_cone.length * 0.5);
        }
    }

    #[test]
    fn explicit_mass_model_places_center_of_mass_below_body_center() {
        let dims = RocketDimensions::default();
        let props = rocket_mass_properties(&dims, &RocketMassModel::default());

        assert!(props.mass.0 > 0.0);
        assert!(props.center_of_mass.0.y < 0.0);
        assert!(props.angular_inertia.principal.cmpgt(Vec3::ZERO).all());
    }

    #[test]
    fn explicit_mass_model_moves_center_of_mass_up_for_longer_cone() {
        let short_cone = RocketDimensions {
            cone_length: 0.05,
            ..RocketDimensions::default()
        };
        let long_cone = RocketDimensions {
            cone_length: 0.20,
            ..RocketDimensions::default()
        };

        let short_props = rocket_mass_properties(&short_cone, &RocketMassModel::default());
        let long_props = rocket_mass_properties(&long_cone, &RocketMassModel::default());

        assert!(long_props.mass.0 > short_props.mass.0);
        assert!(long_props.center_of_mass.0.y > short_props.center_of_mass.0.y);
    }
}
