use bevy::prelude::*;

use bevy::image::CompressedImageFormats;
use bevy::light::{
    CascadeShadowConfigBuilder, FogVolume, NotShadowCaster, NotShadowReceiver, VolumetricFog,
    VolumetricLight,
};
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
    SkyboxEntry {
        name: "Learner Park",
        variants: &[
            (
                "textures/learner_park_cubemap_astc4x4.ktx2",
                CompressedImageFormats::ASTC_LDR,
            ),
            (
                "textures/learner_park_cubemap_etc2.ktx2",
                CompressedImageFormats::ETC2,
            ),
            (
                "textures/learner_park_cubemap.png",
                CompressedImageFormats::NONE,
            ),
        ],
        fog: FogColors {
            base: Color::srgba(0.4, 0.42, 0.48, 1.0),
            extinction: Color::srgb(0.4, 0.42, 0.48),
            inscattering: Color::srgb(0.7, 0.75, 0.82),
            directional: Color::srgba(0.9, 0.85, 0.8, 0.2),
        },
    },
];

#[derive(Resource)]
pub struct Cubemap {
    pub is_loaded: bool,
    image_handle: Handle<Image>,
}

impl Cubemap {
    pub fn image_handle(&self) -> Handle<Image> {
        self.image_handle.clone()
    }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkyRenderMode {
    #[default]
    Cubemap,
    // Currently disabled at runtime: bevy_firework's pipeline is incompatible
    // with Atmosphere bind group bindings (bevyengine/bevy#21784).
    #[allow(dead_code)]
    Atmosphere,
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
    pub lab_skybox_index: usize,
    pub store_skybox_index: usize,
    /// Compresses the ambient brightness curve toward 1.0.
    /// 0.0 = full twilight/night range; 1.0 = always fully lit.
    pub ambient_floor: f32,
}

impl Default for SkyProperties {
    fn default() -> Self {
        let store_index = SKYBOXES
            .iter()
            .position(|s| s.name == "Learner Park")
            .unwrap_or(0);
        Self {
            fog_mode: 0,
            skybox_index: 0,
            skybox_changed: false,
            fog_enabled: false,
            fog_visibility: 150.0,
            volumetrics_enabled: false,
            time_of_day: 10.0,
            day_speed: 60.0,
            lab_skybox_index: 0,
            store_skybox_index: store_index,
            ambient_floor: 0.20,
        }
    }
}

#[derive(Resource)]
pub struct SunDiscSettings {
    pub enabled: bool,
    pub distance: f32,
    pub angular_diameter_deg: f32,
    pub emissive_strength: f32,
}

impl Default for SunDiscSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            distance: 250.0,
            angular_diameter_deg: 0.53,
            emissive_strength: 22_000.0,
        }
    }
}

#[derive(Component)]
pub struct VolumetricFogMarker;

#[derive(Component)]
pub struct SunDiscMarker;

#[derive(Component)]
pub struct SunLightMarker;

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
        SunLightMarker,
    ));
}

pub fn spawn_sun_disc_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<SunDiscSettings>,
) {
    let initial_radius = sun_disc_radius(settings.distance, settings.angular_diameter_deg);
    let sun_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(
            settings.emissive_strength,
            settings.emissive_strength * 0.95,
            settings.emissive_strength * 0.85,
        ),
        emissive_exposure_weight: 1.0,
        unlit: true,
        ..default()
    });
    let sun_mesh = meshes.add(
        Sphere::new(1.0)
            .mesh()
            .ico(4)
            .expect("sun disc sphere should have valid ico subdivisions"),
    );

    commands.spawn((
        Mesh3d(sun_mesh),
        MeshMaterial3d(sun_material),
        Transform::from_scale(Vec3::splat(initial_radius)),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
        SunDiscMarker,
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
    sky_mode: Res<SkyRenderMode>,
    app_state: Res<State<crate::AppState>>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<SunLightMarker>>,
) {
    let dt = time.delta_secs();
    if sky_props.day_speed > 0.0 {
        sky_props.time_of_day = (sky_props.time_of_day + dt / sky_props.day_speed).rem_euclid(24.0);
    } else {
        sky_props.time_of_day = sky_props.time_of_day.rem_euclid(24.0);
    }

    let direction = sun_direction_for_time(sky_props.time_of_day);
    let daylight = direction.y.max(0.0);
    let up = if direction.y.abs() > 0.999 {
        Vec3::X
    } else {
        Vec3::Y
    };

    match app_state.get() {
        crate::AppState::Lab | crate::AppState::Store => {
            // Indoor scenes use a fixed ambient; outdoor day/night doesn't apply.
            ambient_light.brightness = 0.8;
        }
        _ => {
            // Both modes use: ambient = natural + (1 - natural) * floor
            // This compresses the curve toward 1.0 — floor has a visible effect
            // whenever natural < 1.0, including atmosphere mode during daytime
            // (where natural = 0, so ambient = floor directly).
            let natural = match *sky_mode {
                SkyRenderMode::Cubemap => {
                    // Twilight blend: midnight ≈ 0.35, horizon ≈ 0.675, noon = 1.0.
                    let t = (direction.y + 1.0) * 0.5; // 0 at midnight → 1 at noon
                    let s = t * t * (3.0 - 2.0 * t); // smoothstep
                    0.35 + 0.65 * s
                }
                SkyRenderMode::Atmosphere => {
                    // AtmosphereEnvironmentMapLight handles daytime via IBL.
                    // Fade in a moonlit natural floor below the horizon.
                    let night_factor = (-direction.y).clamp(0.0, 1.0);
                    0.15 * night_factor
                }
            };
            let floor_brightness = sky_props.ambient_floor * 3000.0;
            ambient_light.brightness = natural + floor_brightness;
        }
    }

    for (mut transform, mut light) in &mut query {
        *transform = Transform::default().looking_to(-direction, up);
        light.illuminance = match *sky_mode {
            SkyRenderMode::Cubemap => {
                let base = if daylight > 0.0 {
                    light_consts::lux::CLEAR_SUNRISE
                        + (light_consts::lux::RAW_SUNLIGHT - light_consts::lux::CLEAR_SUNRISE)
                            * daylight.powf(0.45)
                } else {
                    light_consts::lux::FULL_MOON_NIGHT
                };
                let floor = sky_props.ambient_floor * light_consts::lux::CLEAR_SUNRISE;
                base.max(floor)
            }
            SkyRenderMode::Atmosphere => {
                light_consts::lux::RAW_SUNLIGHT
            }
        };
    }
}

pub fn update_sun_disc_system(
    sky_props: Res<SkyProperties>,
    sky_mode: Res<SkyRenderMode>,
    settings: Res<SunDiscSettings>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    mut disc_query: Query<
        (
            &mut Transform,
            &MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        With<SunDiscMarker>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let Ok((mut transform, material_handle, mut visibility)) = disc_query.single_mut() else {
        return;
    };

    if *sky_mode != SkyRenderMode::Cubemap || !settings.enabled {
        *visibility = Visibility::Hidden;
        return;
    }

    let sun_direction = sun_direction_for_time(sky_props.time_of_day);
    if sun_direction.y <= -0.05 {
        *visibility = Visibility::Hidden;
        return;
    }

    *visibility = Visibility::Visible;
    let radius = sun_disc_radius(settings.distance, settings.angular_diameter_deg);
    transform.translation = camera_transform.translation() + sun_direction * settings.distance;
    transform.scale = Vec3::splat(radius);

    if let Some(material) = materials.get_mut(&material_handle.0) {
        let warmness = (1.0 - sun_direction.y.clamp(0.0, 1.0)).powf(1.8);
        let r = 1.0;
        let g = 0.95 - 0.28 * warmness;
        let b = 0.85 - 0.45 * warmness;
        let altitude_boost = ((sun_direction.y + 0.05) / 0.4).clamp(0.0, 1.0);
        let emissive_strength = settings.emissive_strength * (0.25 + 0.75 * altitude_boost);
        material.emissive = LinearRgba::rgb(
            r * emissive_strength,
            g * emissive_strength,
            b * emissive_strength,
        );
    }
}

pub fn sun_direction_for_time(time_of_day: f32) -> Vec3 {
    let sun_angle = (time_of_day / 24.0) * TAU - FRAC_PI_2;
    let tilt = 20f32.to_radians();
    Vec3::new(
        sun_angle.cos() * tilt.cos(),
        sun_angle.sin(),
        sun_angle.cos() * tilt.sin(),
    )
    .normalize()
}

fn sun_disc_radius(distance: f32, angular_diameter_deg: f32) -> f32 {
    let half_angle = (angular_diameter_deg.to_radians() * 0.5).max(0.0001);
    distance * half_angle.tan()
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
