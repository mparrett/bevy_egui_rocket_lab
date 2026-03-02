use bevy::prelude::*;

use bevy::image::CompressedImageFormats;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::{
    core_pipeline::Skybox,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use std::f32::consts::PI;

pub struct SkyboxEntry {
    pub name: &'static str,
    pub variants: &'static [(&'static str, CompressedImageFormats)],
}

pub const SKYBOXES: &[SkyboxEntry] = &[
    SkyboxEntry {
        name: "Grasslands Sunset",
        variants: &[
            (
                "textures/grasslands_sunset_cubemap_astc4x4.ktx2",
                CompressedImageFormats::ASTC_LDR,
            ),
            (
                "textures/grasslands_sunset_cubemap_etc2.ktx2",
                CompressedImageFormats::ETC2,
            ),
            (
                "textures/grasslands_sunset_cubemap.png",
                CompressedImageFormats::NONE,
            ),
        ],
    },
    SkyboxEntry {
        name: "Belfast Sunset",
        variants: &[
            (
                "textures/belfast_sunset_cubemap_astc4x4.ktx2",
                CompressedImageFormats::ASTC_LDR,
            ),
            (
                "textures/belfast_sunset_cubemap_etc2.ktx2",
                CompressedImageFormats::ETC2,
            ),
            (
                "textures/belfast_sunset_cubemap.png",
                CompressedImageFormats::NONE,
            ),
        ],
    },
    SkyboxEntry {
        name: "Citrus Orchard",
        variants: &[
            (
                "textures/citrus_orchard_cubemap_astc4x4.ktx2",
                CompressedImageFormats::ASTC_LDR,
            ),
            (
                "textures/citrus_orchard_cubemap_etc2.ktx2",
                CompressedImageFormats::ETC2,
            ),
            (
                "textures/citrus_orchard_cubemap.png",
                CompressedImageFormats::NONE,
            ),
        ],
    },
    SkyboxEntry {
        name: "Bambanani Sunset",
        variants: &[
            (
                "textures/bambanani_sunset_cubemap_astc4x4.ktx2",
                CompressedImageFormats::ASTC_LDR,
            ),
            (
                "textures/bambanani_sunset_cubemap_etc2.ktx2",
                CompressedImageFormats::ETC2,
            ),
            (
                "textures/bambanani_sunset_cubemap.png",
                CompressedImageFormats::NONE,
            ),
        ],
    },
];

#[derive(Resource)]
pub struct Cubemap {
    pub is_loaded: bool,
    image_handle: Handle<Image>,
}

pub const SKY_BLUE_FOG: Color = Color::srgba(0.35, 0.48, 0.66, 1.0);
pub const OVERCAST_FOG: Color = Color::srgba(0.8, 0.844, 1.0, 1.0);
pub const SKY_DIR_LIGHT_COLOR: Color = Color::WHITE;

pub static FOG_MODES: &[usize] = &[0, 1, 2];

#[derive(Resource, Default)]
pub struct SkyProperties {
    pub fog_mode: usize,
    pub skybox_index: usize,
    pub skybox_changed: bool,
}

pub fn setup_sky_system(mut commands: Commands) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb_u8(210, 220, 240),
        brightness: 0.9,
        affects_lightmapped_meshes: true,
    });

    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 3.0,
        maximum_distance: 30.0,
        ..default()
    }
    .build();

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 0.0).with_rotation(Quat::from_rotation_x(-PI / 4.)),
        cascade_shadow_config,
    ));
}

pub fn pick_best_variant<'a>(
    variants: &'a [(&'a str, CompressedImageFormats)],
    supported: CompressedImageFormats,
) -> &'a str {
    variants
        .iter()
        .find(|(_, fmt)| *fmt == CompressedImageFormats::NONE || supported.contains(*fmt))
        .map(|(path, _)| *path)
        .unwrap()
}

fn detect_supported_formats(
    render_device: &Option<Res<bevy::render::renderer::RenderDevice>>,
) -> CompressedImageFormats {
    render_device
        .as_ref()
        .map(|d| CompressedImageFormats::from_features(d.features()))
        .unwrap_or(CompressedImageFormats::NONE)
}

pub fn spawn_regular_sky_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Option<Res<bevy::render::renderer::RenderDevice>>,
) {
    let supported = detect_supported_formats(&render_device);
    let path = pick_best_variant(SKYBOXES[0].variants, supported);

    info!("Loading skybox: {path}");
    let skybox_handle = asset_server.load(path);
    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: skybox_handle.clone(),
    });
}

pub fn cubemap_asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap_opt: Option<ResMut<Cubemap>>,
    mut skybox_query: Query<&mut Skybox>,
    mut sky_props: ResMut<SkyProperties>,
    render_device: Option<Res<bevy::render::renderer::RenderDevice>>,
) {
    if cubemap_opt.is_none() {
        return;
    }
    let cubemap = cubemap_opt.as_mut().unwrap();

    if sky_props.skybox_changed {
        let supported = detect_supported_formats(&render_device);
        let path = pick_best_variant(SKYBOXES[sky_props.skybox_index].variants, supported);
        info!("Loading skybox: {path}");
        cubemap.image_handle = asset_server.load(path);
        cubemap.is_loaded = false;
        sky_props.skybox_changed = false;
    }

    if !cubemap.is_loaded && asset_server.is_loaded(&cubemap.image_handle) {
        info!(
            "Skybox ready: {}",
            SKYBOXES[sky_props.skybox_index].name
        );
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        if image.texture_descriptor.array_layer_count() == 1 {
            let _ = image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skybox_query {
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
        transform.rotate_y(time.delta_secs() * rotate_speed);
    }
}
pub fn toggle_fog_system(
    key_code: Res<ButtonInput<KeyCode>>,
    mut sky_props: ResMut<SkyProperties>,
    mut fog: Query<&mut DistanceFog>,
) {
    if let Ok(mut fog_settings) = fog.single_mut() {
        if key_code.just_pressed(KeyCode::KeyF) {
            debug!("Toggle fog alpha");
            let a = fog_settings.color.alpha();
            fog_settings.color = fog_settings.color.with_alpha(1.0 - a);
        }

        if key_code.just_pressed(KeyCode::KeyL) {
            debug!("Toggle fog lighting alpha");
            let a = fog_settings.directional_light_color.alpha();
            fog_settings.directional_light_color =
                fog_settings.directional_light_color.with_alpha(0.5 - a);
        }

        if key_code.just_pressed(KeyCode::KeyT) {
            sky_props.fog_mode = (sky_props.fog_mode + 1) % FOG_MODES.len();
            debug!("Toggle fog type to {}", sky_props.fog_mode);
            if sky_props.fog_mode == 0 {
                *fog_settings = DistanceFog::default();
            } else if sky_props.fog_mode == 1 {
                *fog_settings = DistanceFog {
                    color: SKY_BLUE_FOG,
                    directional_light_color: SKY_DIR_LIGHT_COLOR,
                    directional_light_exponent: 30.0,
                    falloff: FogFalloff::from_visibility_colors(
                        15.0,
                        Color::srgb(0.35, 0.5, 0.66),
                        Color::srgb(0.8, 0.844, 1.0),
                    ),
                };
            } else if sky_props.fog_mode == 2 {
                *fog_settings = DistanceFog {
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
