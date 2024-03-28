use bevy::prelude::*;

use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        texture::CompressedImageFormats,
    },
};
use std::f32::consts::PI;

pub const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[
    (
        "textures/Ryfjallet_cubemap.png",
        CompressedImageFormats::NONE,
    ),
    (
        "textures/Ryfjallet_cubemap_etc2.ktx2",
        CompressedImageFormats::ETC2,
    ),
    (
        "textures/Ryfjallet_cubemap_astc4x4.ktx2",
        CompressedImageFormats::ASTC_LDR,
    ),
];

pub const CUBEMAP_IDX: usize = 2;

#[derive(Resource)]
pub struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}
use bevy::pbr::CascadeShadowConfigBuilder;

pub const SKY_BLUE_FOG: Color = Color::rgba(0.35, 0.48, 0.66, 1.0);
pub const OVERCAST_FOG: Color = Color::rgba(0.8, 0.844, 1.0, 1.0);
//pub const SKY_DIR_LIGHT_COLOR = Color::rgba(1.0, 0.95, 0.85, 0.5);
pub const SKY_DIR_LIGHT_COLOR: Color = Color::WHITE;

pub static FOG_MODES: &[usize] = &[0, 1, 2];

#[derive(Resource, Default)]
pub struct SkyProperties {
    pub fog_mode: usize,
}

pub fn setup_sky_system(mut commands: Commands) {
    // ambient light, added with skymap
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate color and brightness to match
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(210, 220, 240),
        brightness: 0.9, // 1.0,
    });

    // Configure a properly scaled cascade shadow map for this scene
    // (defaults are too large, mesh units are in km)
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 3.0, // 5.0
        maximum_distance: 30.0,       // 1000.0
        ..default()
    }
    .build();

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 2.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
        cascade_shadow_config,
        ..default()
    });
}

pub fn spawn_regular_sky_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let skybox_handle = asset_server.load(CUBEMAPS[CUBEMAP_IDX].0);
    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: CUBEMAP_IDX,
        image_handle: skybox_handle.clone(),
    });
}

pub fn cubemap_asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap_opt: Option<ResMut<Cubemap>>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if cubemap_opt.is_none() {
        return;
    }
    let cubemap = cubemap_opt.as_mut().unwrap();

    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle) == LoadState::Loaded {
        info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}

pub fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    let rotate_speed = 0.3;
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() * rotate_speed);
    }
}
/*
https://github.com/claudijo/pirate-sea-jam/blob/main/src/sky_box/systems.rs
Pirate sky box
*/
#[derive(Resource)]
pub struct FocalPoint(pub Vec3);
impl Default for FocalPoint {
    fn default() -> Self {
        FocalPoint(Vec3::new(0., 0., 0.))
    }
}

#[derive(Component)]
pub struct Sky;

use bevy::pbr::{NotShadowCaster, PbrBundle, StandardMaterial};

const WORLD_TILE_SIZE: f32 = 100.;

#[allow(dead_code)]
pub fn spawn_simple_sky_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Simple sky box; from pirate demo
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1., 1., 1.))),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("a5cddf").unwrap(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(WORLD_TILE_SIZE * 10.)),
            ..default()
        },
        Sky,
        NotShadowCaster,
    ));
}

// From pirate demo;     I suppose this is used to sync with a character position, not sure
//pub fn sync_sky_box_center_offset(
//    focal_point: Res<FocalPoint>,
//    mut sky_box_query: Query<&mut Transform, With<Sky>>,
//) {
//    for mut transform in &mut sky_box_query {
//        transform.translation = focal_point.0;
//    }
//}

pub fn toggle_fog_system(
    key_code: Res<ButtonInput<KeyCode>>,
    mut sky_props: ResMut<SkyProperties>,
    mut fog: Query<&mut FogSettings>,
) {
    if let Ok(mut fog_settings) = fog.get_single_mut() {
        if key_code.just_pressed(KeyCode::KeyF) {
            println!("Toggle fog alpha");
            let a = fog_settings.color.a();
            fog_settings.color.set_a(1.0 - a);
        }

        if key_code.just_pressed(KeyCode::KeyL) {
            println!("Toggle fog lighting alpha");
            let a = fog_settings.directional_light_color.a();
            fog_settings.directional_light_color.set_a(0.5 - a);
        }

        if key_code.just_pressed(KeyCode::KeyT) {
            sky_props.fog_mode = (sky_props.fog_mode + 1) % FOG_MODES.len();
            println!("Toggle fog type to {}", sky_props.fog_mode);
            if sky_props.fog_mode == 0 {
                *fog_settings = FogSettings::default();
            } else if sky_props.fog_mode == 1 {
                *fog_settings = FogSettings {
                    color: SKY_BLUE_FOG,
                    directional_light_color: SKY_DIR_LIGHT_COLOR,
                    directional_light_exponent: 30.0,
                    falloff: FogFalloff::from_visibility_colors(
                        15.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                        Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                        Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
                    ),
                };
            } else if sky_props.fog_mode == 2 {
                *fog_settings = FogSettings {
                    color: OVERCAST_FOG,
                    falloff: FogFalloff::Linear {
                        start: 80.0,
                        end: 160.0,
                    },
                    ..default()
                };
            }
        }
    }
}
