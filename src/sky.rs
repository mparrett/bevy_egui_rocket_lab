use bevy::prelude::*;

use bevy::image::CompressedImageFormats;
use bevy::light::{CascadeShadowConfigBuilder, FogVolume, VolumetricFog, VolumetricLight};
use bevy::{
    core_pipeline::Skybox,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub struct FogColors {
    pub base: Color,
    pub extinction: Color,
    pub inscattering: Color,
    pub directional: Color,
}

pub struct SkyboxEntry {
    pub name: &'static str,
    pub variants: &'static [(&'static str, CompressedImageFormats)],
    pub fog: FogColors,
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
        fog: FogColors {
            base: Color::srgba(0.4, 0.45, 0.35, 1.0),
            extinction: Color::srgb(0.4, 0.45, 0.35),
            inscattering: Color::srgb(0.85, 0.75, 0.55),
            directional: Color::srgba(1.0, 0.85, 0.6, 0.3),
        },
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
        fog: FogColors {
            base: Color::srgba(0.45, 0.38, 0.42, 1.0),
            extinction: Color::srgb(0.45, 0.38, 0.42),
            inscattering: Color::srgb(0.9, 0.65, 0.5),
            directional: Color::srgba(1.0, 0.8, 0.55, 0.3),
        },
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
        fog: FogColors {
            base: Color::srgba(0.5, 0.55, 0.45, 1.0),
            extinction: Color::srgb(0.45, 0.5, 0.4),
            inscattering: Color::srgb(0.8, 0.82, 0.7),
            directional: Color::srgba(1.0, 0.95, 0.8, 0.3),
        },
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
        fog: FogColors {
            base: Color::srgba(0.5, 0.4, 0.35, 1.0),
            extinction: Color::srgb(0.5, 0.4, 0.35),
            inscattering: Color::srgb(0.9, 0.7, 0.5),
            directional: Color::srgba(1.0, 0.8, 0.5, 0.3),
        },
    },
];

#[derive(Resource)]
pub struct Cubemap {
    pub is_loaded: bool,
    image_handle: Handle<Image>,
}

#[derive(Resource)]
pub struct SkyProperties {
    pub fog_mode: usize,
    pub skybox_index: usize,
    pub skybox_changed: bool,
    pub fog_enabled: bool,
    pub fog_visibility: f32,
    pub volumetrics_enabled: bool,
    pub time_of_day: f32,
    pub day_speed: f32,
}

impl Default for SkyProperties {
    fn default() -> Self {
        Self {
            fog_mode: 0,
            skybox_index: 0,
            skybox_changed: false,
            fog_enabled: false,
            fog_visibility: 150.0,
            volumetrics_enabled: false,
            time_of_day: 17.0,
            day_speed: 60.0,
        }
    }
}

#[derive(Component)]
pub struct VolumetricFogMarker;

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

pub fn sync_volumetrics_system(
    mut commands: Commands,
    sky_props: Res<SkyProperties>,
    camera_query: Query<(Entity, Option<&VolumetricFog>), With<Camera3d>>,
    light_query: Query<(Entity, Option<&VolumetricLight>), With<DirectionalLight>>,
    volume_query: Query<Entity, With<VolumetricFogMarker>>,
) {
    if !sky_props.is_changed() {
        return;
    }

    let enable = sky_props.volumetrics_enabled;

    if let Ok((camera_entity, volumetric_fog)) = camera_query.single() {
        if enable && volumetric_fog.is_none() {
            commands.entity(camera_entity).insert(VolumetricFog {
                ambient_intensity: 0.0,
                ..default()
            });
        } else if !enable && volumetric_fog.is_some() {
            commands.entity(camera_entity).remove::<VolumetricFog>();
        }
    }

    for (light_entity, volumetric_light) in &light_query {
        if enable && volumetric_light.is_none() {
            commands.entity(light_entity).insert(VolumetricLight);
        } else if !enable && volumetric_light.is_some() {
            commands.entity(light_entity).remove::<VolumetricLight>();
        }
    }

    if enable {
        if volume_query.is_empty() {
            commands.spawn((
                FogVolume {
                    density_factor: 0.02,
                    ..default()
                },
                Transform::from_scale(Vec3::splat(200.0)),
                VolumetricFogMarker,
            ));
        }
    } else {
        for entity in &volume_query {
            commands.entity(entity).despawn();
        }
    }
}

pub fn pick_best_variant<'a>(
    variants: &'a [(&'a str, CompressedImageFormats)],
    supported: CompressedImageFormats,
) -> &'a str {
    let Some((fallback_path, _)) = variants.first() else {
        panic!("Skybox variant list must not be empty");
    };

    if let Some((path, _)) = variants
        .iter()
        .find(|(_, fmt)| *fmt == CompressedImageFormats::NONE || supported.contains(*fmt))
    {
        path
    } else {
        warn!(
            "No supported compressed skybox format found; falling back to {}",
            fallback_path
        );
        fallback_path
    }
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
    let Some(cubemap) = cubemap_opt.as_mut() else {
        return;
    };

    if sky_props.skybox_changed {
        let supported = detect_supported_formats(&render_device);
        let path = pick_best_variant(SKYBOXES[sky_props.skybox_index].variants, supported);
        info!("Loading skybox: {path}");
        cubemap.image_handle = asset_server.load(path);
        cubemap.is_loaded = false;
        sky_props.skybox_changed = false;
    }

    if !cubemap.is_loaded && asset_server.is_loaded(&cubemap.image_handle) {
        info!("Skybox ready: {}", SKYBOXES[sky_props.skybox_index].name);
        let Some(image) = images.get_mut(&cubemap.image_handle) else {
            return;
        };
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
    mut sky_props: ResMut<SkyProperties>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    let dt = time.delta_secs();
    if sky_props.day_speed > 0.0 {
        sky_props.time_of_day += dt / sky_props.day_speed;
        if sky_props.time_of_day >= 24.0 {
            sky_props.time_of_day -= 24.0;
        }
    }

    let sun_angle = (sky_props.time_of_day / 24.0) * TAU - FRAC_PI_2;
    let tilt = 20f32.to_radians();

    let direction = Vec3::new(
        sun_angle.cos() * tilt.cos(),
        sun_angle.sin(),
        sun_angle.cos() * tilt.sin(),
    )
    .normalize();

    for mut transform in &mut query {
        *transform = Transform::default().looking_to(-direction, Vec3::Y);
    }
}
pub fn make_atmospheric_fog(visibility: f32, skybox_index: usize) -> DistanceFog {
    let fog = &SKYBOXES[skybox_index].fog;
    DistanceFog {
        color: fog.base,
        directional_light_color: fog.directional,
        directional_light_exponent: 30.0,
        falloff: FogFalloff::from_visibility_colors(visibility, fog.extinction, fog.inscattering),
    }
}

pub fn apply_fog_mode(
    fog_settings: &mut DistanceFog,
    mode: usize,
    visibility: f32,
    skybox_index: usize,
) {
    match mode {
        1 => *fog_settings = make_atmospheric_fog(visibility, skybox_index),
        2 => *fog_settings = make_atmospheric_fog(visibility * 0.375, skybox_index),
        _ => {
            fog_settings.color = fog_settings.color.with_alpha(0.0);
            fog_settings.directional_light_color =
                fog_settings.directional_light_color.with_alpha(0.0);
        }
    }
}
