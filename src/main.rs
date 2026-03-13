use bevy::{
    camera::{CameraOutputMode, Exposure, Viewport, visibility::RenderLayers},
    window::{CursorIcon, SystemCursorIcon},
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::system::SystemParam,
    image::{CompressedImageFormats, ImageAddressMode, ImageSamplerDescriptor},
    input::common_conditions::input_toggle_active,
    light::AtmosphereEnvironmentMapLight,
    math::primitives::Cylinder,
    pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
    render::{render_resource::BlendState, view::Hdr},
};
use bevy_firework::plugin::ParticleSystemPlugin;

use avian3d::prelude::*;
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use egui::Key;
use particles::RocketParticlesPlugin;
use sky::{SkyProperties, SkyRenderMode, SunDiscSettings};

use crate::{
    camera::{
        AuxCamKind, CameraProperties, DroneCamMarker, DRONE_CAM_FOV_DEGREES,
        DRONE_CAM_POSITION, DroneDistance, DroneWaypoint, FollowMode, INITIAL_CAMERA_POS,
        RocketCamMarker, SceneCameraState, ZOOM_LEVELS, mouse_orbit_system, spring_to_target,
        update_camera_transform_system, update_camera_zoom_perspective_system,
    },
    cone::Cone,
    fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter},
    ground::setup_ground_system,
    physics::{ForceTimer, get_timer_id, lock_all_axes, update_forces_system},
    rocket::{
        ColorPreset, FinMarker, RocketBody, RocketCone, RocketDimensions, RocketFlightParameters,
        RocketMarker, RocketMassModel, RocketMaterial, RocketState, RocketStateEnum,
        create_rocket_fin_pbr_bundles,
        rocket_mass_properties, spawn_rocket_system,
    },
    sky::{
        Cubemap, SKYBOXES, animate_light_direction, apply_fog_mode, cubemap_asset_loaded,
        pick_best_variant, setup_sky_system, spawn_regular_sky_map, spawn_sun_disc_system,
        sync_volumetrics_system, update_sun_disc_system,
    },
};

mod camera;
mod canopy;
mod cone;
mod drag;
mod fin;
mod fps;
mod ground;
mod inventory;
mod menu;
mod parachute;
mod particles;
mod physics;
mod profiling;
mod rendering;
mod rocket;
mod save;
mod scene;
mod sky;
mod wind;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    Lab,
    Launch,
    Store,
}

fn in_gameplay(state: Res<State<AppState>>) -> bool {
    matches!(
        state.get(),
        AppState::Lab | AppState::Launch | AppState::Store
    )
}

#[derive(Message)]
struct LaunchEvent;

#[derive(Message)]
struct DownedEvent;

#[derive(Message, Default)]
struct ResetEvent;

#[derive(Message, Default)]
struct RocketGeometryChangedEvent;

#[derive(Component, Default)]
struct ScoreMarker;

#[derive(Component, Default)]
struct WindHudMarker;

#[derive(Component, Default)]
struct WindIconMarker;

#[derive(Component)]
struct NavButton(AppState);

#[derive(Component)]
struct CameraModeButton;

#[derive(Component)]
struct CameraModeLabel;

#[derive(Component)]
struct CountdownDisplayMarker;

#[derive(Component)]
struct CountdownOverlay;

#[derive(Resource)]
struct CountdownTimer {
    timer: Timer,
    remaining: u8,
}

impl CountdownTimer {
    fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            remaining: 3,
        }
    }
}

#[derive(Resource)]
pub struct AudioSettings {
    pub music_enabled: bool,
    pub sfx_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_enabled: true,
            sfx_enabled: true,
        }
    }
}

#[derive(Component)]
pub struct LabMusicMarker;

#[derive(Component)]
pub struct LaunchMusicMarker;

#[derive(Component)]
struct LoadingOverlay;

fn setup_loading_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(i32::MAX - 1),
        LoadingOverlay,
    ));
}

fn check_loading_complete(
    mut commands: Commands,
    cubemap: Option<Res<Cubemap>>,
    overlay: Query<Entity, With<LoadingOverlay>>,
) {
    if cubemap.is_some_and(|c| c.is_loaded) {
        for entity in &overlay {
            commands.entity(entity).despawn();
        }
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin {
        default_sampler: ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            min_filter: bevy::image::ImageFilterMode::Linear,
            mag_filter: bevy::image::ImageFilterMode::Linear,
            mipmap_filter: bevy::image::ImageFilterMode::Linear,
            anisotropy_clamp: 4,
            ..default()
        },
    }));

    app.add_message::<LaunchEvent>();
    app.add_message::<DownedEvent>();
    app.add_message::<ResetEvent>();
    app.add_message::<RocketGeometryChangedEvent>();
    app.add_message::<parachute::DeployParachuteEvent>();

    app.add_plugins(ParticleSystemPlugin::default())
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin)
        .insert_resource(SkyProperties::default())
        .insert_resource(SkyRenderMode::default())
        .insert_resource(SunDiscSettings::default())
        .insert_resource(Gravity(Vec3::NEG_Y * 9.81 * 1.0))
        .add_plugins(EguiPlugin::default())
        .insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        .add_plugins(RocketParticlesPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(profiling::ProfilingPlugin)
        .register_type::<ForceTimer>()
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .init_state::<AppState>()
        .add_plugins(menu::MenuPlugin)
        .add_plugins(scene::ScenePlugin)
        .init_resource::<RocketDimensions>()
        .init_resource::<RocketMassModel>()
        .init_resource::<RocketFlightParameters>()
        .init_resource::<CameraProperties>()
        .init_resource::<CameraDebugOpen>()
        .init_resource::<RocketState>()
        .init_resource::<AudioSettings>()
        .init_resource::<save::SaveState>()
        .init_resource::<save::PlayerBalance>()
        .init_resource::<save::GameMode>()
        .init_resource::<save::OwnedMaterials>()
        .init_resource::<save::RocketCamOwned>()
        .init_resource::<inventory::Inventory>()
        .init_resource::<inventory::OwnedMotorSizes>()
        .init_resource::<inventory::OwnedTubeTypes>()
        .init_resource::<inventory::OwnedNoseconeTypes>()
        .init_resource::<inventory::EquippedLoadout>()
        .init_resource::<inventory::PlayerExperience>()
        .init_resource::<wind::WindProperties>()
        .init_resource::<parachute::ParachuteConfig>()
        .init_resource::<SceneCameraState>()
        .add_systems(
            Startup,
            (
                setup_ground_system,
                setup_camera_system,
                spawn_rocket_system,
                setup_sky_system,
                setup_fps_counter,
                spawn_music,
                setup_loading_overlay,
                spawn_sun_disc_system,
                disable_physics_debug,
            ),
        )
        .add_systems(
            Startup,
            (spawn_rocket_cam_system, spawn_drone_cam_system).after(spawn_rocket_system),
        )
        .add_systems(OnEnter(AppState::Launch), setup_launch_hud)
        .add_systems(OnExit(AppState::Launch), (disable_aux_cams_on_exit, parachute::cleanup_parachute_on_scene_exit, cleanup_countdown_on_exit))
        .add_systems(OnEnter(AppState::Lab), setup_lab_hud)
        .add_systems(OnEnter(AppState::Store), setup_store_hud)
        .add_systems(
            Update,
            (
                handle_nav_button_clicks,
                update_store_balance_text.run_if(in_state(AppState::Store)),
                camera_mode_button_system.run_if(in_state(AppState::Launch)),
                update_camera_mode_label.run_if(in_state(AppState::Launch)),
            )
                .run_if(in_gameplay),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (ui_system, init_egui_ui_input_system, camera_debug_system).run_if(in_gameplay),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                do_launch_system,
                toggle_aux_cam_system,
            )
                .run_if(in_state(AppState::Launch)),
        )
        .add_systems(
            Update,
            (update_rocket_dimensions_system, on_reset_event.after(parachute::cleanup_parachute_system), sync_equipped_loadout).run_if(in_gameplay),
        )
        .add_systems(
            Update,
            (
                countdown_tick_system,
                on_launch_event,
                on_launch_audio_event,
                detect_landing_from_collision_system,
                on_crash_event,
                update_stats_system,
                update_wind_hud_system,
                update_countdown_display,
                wind::update_wind_system,
                drone_cam_track_rocket_system,
            )
                .run_if(in_state(AppState::Launch)),
        )
        .add_systems(
            Update,
            (
                parachute::auto_deploy_parachute_system,
                parachute::deploy_parachute_system,
                parachute::animate_canopy_system,
                parachute::cleanup_parachute_system,
            )
                .run_if(in_state(AppState::Launch)),
        )
        .add_systems(
            Update,
            (
                fps_text_update_system,
                fps_counter_showhide,
                music_crossfade_system,
                toggle_physics_debug,
                aux_cam_viewport_resize_system,
                sync_aux_cam_kind_system,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                (rocket_position_system, update_camera_transform_system).chain(),
                parachute::update_shock_cord_system,
                parachute::update_shroud_lines_system,
                update_camera_zoom_perspective_system,
            )
                .run_if(in_gameplay)
                .after(PhysicsSystems::Writeback)
                .before(TransformSystems::Propagate),
        );
    app.add_systems(
        FixedPostUpdate,
        (
            update_forces_system,
            wind::apply_wind_force_system,
            drag::apply_aerodynamic_drag_system,
            drag::apply_cone_drag_system,
            parachute::parachute_drag_system,
        )
            .in_set(PhysicsSystems::First)
            .run_if(in_state(AppState::Launch)),
    );

    app.add_systems(Startup, spawn_regular_sky_map);
    app.add_systems(Update, (cubemap_asset_loaded, check_loading_complete));
    app.add_systems(
        Update,
        (
            sync_sky_render_mode_system,
            (animate_light_direction, update_sun_disc_system).chain(),
            sync_volumetrics_system,
        )
            .run_if(in_state(AppState::Launch)),
    );
    app.add_systems(
        Update,
        (adjust_time_scale, mouse_orbit_system).run_if(in_gameplay),
    );

    app.run();
}

fn adjust_time_scale(
    mut contexts: EguiContexts,
    mut time: ResMut<Time<Virtual>>,
    input: Res<ButtonInput<KeyCode>>,
) -> Result {
    if contexts.ctx_mut()?.wants_keyboard_input() {
        return Ok(());
    }
    let slowmo_active = input.pressed(KeyCode::Backquote);
    time.set_relative_speed(if slowmo_active { 0.05 } else { 1.0 });
    Ok(())
}

fn disable_physics_debug(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<PhysicsGizmos>();
    config.enabled = false;
}

fn toggle_physics_debug(
    mut contexts: EguiContexts,
    input: Res<ButtonInput<KeyCode>>,
    mut config_store: ResMut<GizmoConfigStore>,
) -> Result {
    if contexts.ctx_mut()?.wants_keyboard_input() {
        return Ok(());
    }
    if input.just_pressed(KeyCode::F10) {
        let (config, _) = config_store.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
    }
    Ok(())
}

fn sync_sky_render_mode_system(
    sky_mode: Res<SkyRenderMode>,
    sky_props: Res<SkyProperties>,
    cubemap: Option<Res<Cubemap>>,
    asset_server: Res<AssetServer>,
    mut scattering_media: ResMut<Assets<ScatteringMedium>>,
    mut cached_medium: Local<Option<Handle<ScatteringMedium>>>,
    mut commands: Commands,
    mut camera_query: Query<
        (
            Entity,
            Option<&Skybox>,
            Option<&Atmosphere>,
            &mut Tonemapping,
        ),
        (With<Camera3d>, Without<RocketCamMarker>, Without<DroneCamMarker>, Without<camera::EguiOverlayCam>),
    >,
) {
    if !sky_mode.is_changed() {
        return;
    }

    let Ok((camera, skybox, atmosphere, mut tonemapping)) = camera_query.single_mut() else {
        return;
    };

    match *sky_mode {
        SkyRenderMode::Cubemap => {
            *tonemapping = Tonemapping::TonyMcMapface;
            commands.entity(camera).remove::<(
                Exposure,
                Atmosphere,
                AtmosphereSettings,
                AtmosphereEnvironmentMapLight,
            )>();

            if skybox.is_none() {
                let skybox_image =
                    cubemap
                        .as_ref()
                        .map(|c| c.image_handle())
                        .unwrap_or_else(|| {
                            let path = pick_best_variant(
                                SKYBOXES[sky_props.skybox_index].variants,
                                CompressedImageFormats::NONE,
                            );
                            asset_server.load(path)
                        });
                commands.entity(camera).insert(Skybox {
                    image: skybox_image,
                    brightness: 1000.0,
                    ..default()
                });
            }
        }
        SkyRenderMode::Atmosphere => {
            // AcesFitted is what the Bevy atmosphere example uses; it handles
            // HDR sky highlights better than TonyMcMapface for this pipeline.
            *tonemapping = Tonemapping::AcesFitted;
            commands.entity(camera).insert(Exposure { ev100: 13.0 });
            if skybox.is_some() {
                commands.entity(camera).remove::<Skybox>();
            }

            if atmosphere.is_none() {
                let medium = cached_medium
                    .get_or_insert_with(|| scattering_media.add(ScatteringMedium::default()))
                    .clone();
                commands.entity(camera).insert((
                    Atmosphere::earthlike(medium),
                    AtmosphereSettings::default(),
                    AtmosphereEnvironmentMapLight::default(),
                ));
            }
        }
    }
}

fn init_egui_ui_input_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut app_exit: MessageWriter<AppExit>,
    mut reset: MessageWriter<ResetEvent>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    countdown: Option<Res<CountdownTimer>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    if ctx.input(|i| i.key_pressed(Key::Tab)) {
        let shift = ctx.input(|i| i.modifiers.shift);
        match (app_state.get(), shift) {
            (AppState::Lab, false) => next_state.set(AppState::Launch),
            (AppState::Lab, true) => next_state.set(AppState::Store),
            (AppState::Launch, _) => next_state.set(AppState::Lab),
            (AppState::Store, _) => next_state.set(AppState::Lab),
            _ => {}
        }
    }

    if ctx.wants_keyboard_input() {
        return Ok(());
    }

    if ctx.input(|i| i.key_pressed(Key::Q)) {
        app_exit.write(AppExit::Success);
    }

    if ctx.input(|i| i.key_pressed(Key::R)) {
        if countdown.is_some() {
            info!("Countdown cancelled");
            commands.remove_resource::<CountdownTimer>();
        }
        info!("Resetting rocket state");
        reset.write_default();
    }

    Ok(())
}

fn do_launch_system(
    mut contexts: EguiContexts,
    mut camera_properties: ResMut<CameraProperties>,
    mut commands: Commands,
    rocket_state: Res<RocketState>,
    countdown: Option<Res<CountdownTimer>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if ctx.wants_keyboard_input() {
        return Ok(());
    }
    let shift_held = ctx.input(|i| i.modifiers.shift);
    let arrow_left = ctx.input(|i| i.key_down(Key::ArrowLeft));
    let arrow_right = ctx.input(|i| i.key_down(Key::ArrowRight));
    let arrow_up = ctx.input(|i| i.key_down(Key::ArrowUp));
    let arrow_down = ctx.input(|i| i.key_down(Key::ArrowDown));

    if ctx.input(|i| i.key_pressed(Key::Z)) {
        camera_properties.zoom_index = (camera_properties.zoom_index + 1) % ZOOM_LEVELS.len();
        camera_properties.zoom = ZOOM_LEVELS[camera_properties.zoom_index];
    }

    if arrow_left {
        camera_properties.orbit_angle_degrees -= 0.5;
        if camera_properties.orbit_angle_degrees < 0.0 {
            camera_properties.orbit_angle_degrees = 360.0;
        }
    } else if arrow_right {
        camera_properties.orbit_angle_degrees += 0.5;
        if camera_properties.orbit_angle_degrees > 360.0 {
            camera_properties.orbit_angle_degrees = 0.0;
        }
    }

    let plus_held = ctx.input(|i| i.key_down(Key::Plus) || i.key_down(Key::Equals));
    let minus_held = ctx.input(|i| i.key_down(Key::Minus));

    if plus_held {
        camera_properties.desired_translation.y += 0.1;
    } else if minus_held {
        camera_properties.desired_translation.y -= 0.1;
    }

    if arrow_up {
        if shift_held {
            let delta_to_target = camera_properties.desired_translation - camera_properties.target;
            let increment = 0.05;
            camera_properties.desired_translation.x -= increment * delta_to_target.x;
            camera_properties.desired_translation.z -= increment * delta_to_target.z;
        } else {
            camera_properties.fixed_distance =
                (camera_properties.fixed_distance - 0.1).clamp(0.0, 50.0);
        }
    } else if arrow_down {
        if shift_held {
            let delta_to_target = camera_properties.desired_translation - camera_properties.target;
            let increment = 0.05;
            camera_properties.desired_translation.x += increment * delta_to_target.x;
            camera_properties.desired_translation.z += increment * delta_to_target.z;
        } else {
            camera_properties.fixed_distance =
                (camera_properties.fixed_distance + 0.1).clamp(0.0, 50.0);
        }
    }

    if ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space))
        && rocket_state.state == RocketStateEnum::Initial
        && countdown.is_none()
    {
        info!("Begin countdown!");
        commands.insert_resource(CountdownTimer::new());
    }

    Ok(())
}

fn countdown_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    countdown: Option<ResMut<CountdownTimer>>,
    mut launch_event_writer: MessageWriter<LaunchEvent>,
) {
    let Some(mut countdown) = countdown else {
        return;
    };
    countdown.timer.tick(time.delta());
    if countdown.timer.just_finished() {
        if countdown.remaining > 1 {
            countdown.remaining -= 1;
        } else {
            info!("Countdown complete — launching!");
            launch_event_writer.write(LaunchEvent);
            commands.remove_resource::<CountdownTimer>();
        }
    }
}

fn cleanup_countdown_on_exit(mut commands: Commands) {
    commands.remove_resource::<CountdownTimer>();
}

fn update_countdown_display(
    countdown: Option<Res<CountdownTimer>>,
    mut overlay_query: Query<&mut Visibility, With<CountdownOverlay>>,
    mut text_query: Query<&mut Text, With<CountdownDisplayMarker>>,
) {
    let Ok(mut visibility) = overlay_query.single_mut() else {
        return;
    };
    match countdown {
        Some(cd) => {
            *visibility = Visibility::Visible;
            if let Ok(mut text) = text_query.single_mut() {
                **text = cd.remaining.to_string();
            }
        }
        None => {
            *visibility = Visibility::Hidden;
        }
    }
}

fn rocket_position_system(
    rocket_query: Query<(&Transform, &LinearVelocity), (With<RocketMarker>, Without<Camera>)>,
    mut camera_properties: ResMut<CameraProperties>,
    mut rocket_state: ResMut<RocketState>,
    app_state: Res<State<AppState>>,
) {
    let Ok((transform, velocity)) = rocket_query.single() else {
        return;
    };
    if *app_state.get() == AppState::Launch && camera_properties.follow_mode != FollowMode::FreeLook
    {
        camera_properties.target = transform.translation;
    }
    if rocket_state.state == RocketStateEnum::Initial {
        rocket_state.launch_origin_y = transform.translation.y;
    }

    if matches!(
        rocket_state.state,
        RocketStateEnum::Launched | RocketStateEnum::Descending
    ) {
        let current_altitude = (transform.translation.y - rocket_state.launch_origin_y).max(0.0);
        rocket_state.max_height = current_altitude.max(rocket_state.max_height);
        rocket_state.max_velocity = velocity.length().max(rocket_state.max_velocity);
    }
}

fn on_crash_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut crash_reader: MessageReader<DownedEvent>,
) {
    for _event in crash_reader.read() {
        info!("Crash event");
        if !audio_settings.sfx_enabled {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(asset_server.load("audio/impact_wood.ogg")),
            PlaybackSettings::DESPAWN,
        ));
    }
}

#[derive(SystemParam)]
struct LaunchSaveParams<'w> {
    save_state: Res<'w, save::SaveState>,
    balance: Res<'w, save::PlayerBalance>,
    owned_materials: Res<'w, save::OwnedMaterials>,
    rocket_cam_owned: Res<'w, save::RocketCamOwned>,
    owned_motor_sizes: Res<'w, inventory::OwnedMotorSizes>,
    owned_tube_types: Res<'w, inventory::OwnedTubeTypes>,
    owned_nosecone_types: Res<'w, inventory::OwnedNoseconeTypes>,
    experience: Res<'w, inventory::PlayerExperience>,
}

#[allow(unused_variables)]
fn on_launch_event(
    mut launch_events: MessageReader<LaunchEvent>,
    mut locked_axes: Query<&mut LockedAxes, With<RocketMarker>>,
    mut rocket_state: ResMut<RocketState>,
    mut commands: Commands,
    mut rocket_flight_parameters: ResMut<RocketFlightParameters>,
    parachute_config: Res<parachute::ParachuteConfig>,
    mut rocket_query: Query<(Entity, &Transform), (With<RocketMarker>, Without<Camera>)>,
    mut inv: ResMut<inventory::Inventory>,
    equipped: Res<inventory::EquippedLoadout>,
    game_mode: Res<save::GameMode>,
    mut mass_model: ResMut<RocketMassModel>,
    lsp: LaunchSaveParams,
) {
    for _ in launch_events.read() {
        info!("Launch event");
        if rocket_state.state == RocketStateEnum::Launched {
            info!("Rocket already launched");
            return;
        }

        if *game_mode == save::GameMode::Gameplay {
            if !inv.consume_motor(equipped.motor) {
                warn!(
                    "No {} motors remaining — launch rejected",
                    equipped.motor.label()
                );
                return;
            }

            let params = equipped.motor.flight_parameters();
            rocket_flight_parameters.force = params.force;
            rocket_flight_parameters.duration = params.duration;
            equipped.motor.apply_to_mass_model(&mut mass_model);

            #[cfg(not(target_arch = "wasm32"))]
            {
                let meta = save::build_player_meta(
                    &lsp.save_state.player_name,
                    lsp.balance.0,
                    &lsp.owned_materials.0,
                    lsp.rocket_cam_owned.0,
                    &inv,
                    &lsp.owned_motor_sizes.0,
                    &lsp.owned_tube_types.0,
                    &lsp.owned_nosecone_types.0,
                    lsp.experience.0,
                );
                let _ = save::save_player_meta(&meta);
            }
        }

        for mut locked_axes in locked_axes.iter_mut() {
            debug!("Unlock axes");
            *locked_axes = LockedAxes::new();
        }

        let Ok((rocket_ent, transform)) = rocket_query.single_mut() else {
            warn!("Launch requested but no rocket entity is available");
            continue;
        };
        rocket_state.state = RocketStateEnum::Launched;
        rocket_state.max_height = 0.0;
        rocket_state.max_velocity = 0.0;
        rocket_state.launch_origin_y = transform.translation.y;

        let force_timer = ForceTimer {
            id: get_timer_id(),
            timer: Timer::from_seconds(rocket_flight_parameters.duration, TimerMode::Once),
            force: Some(Vec3::Y * rocket_flight_parameters.force),
            torque: None,
            sync_rotation_with_entity: true,
        };
        let ejection_timer = parachute::EjectionTimer {
            timer: Timer::from_seconds(
                rocket_flight_parameters.duration + parachute_config.deploy_delay,
                TimerMode::Once,
            ),
        };
        commands
            .entity(rocket_ent)
            .insert((force_timer, ejection_timer));
    }
}

fn on_launch_audio_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut launch_events: MessageReader<LaunchEvent>,
) {
    for _ in launch_events.read() {
        if !audio_settings.sfx_enabled {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(asset_server.load("audio/air-rushes-out-fast-long.ogg")),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn on_reset_event(
    mut commands: Commands,
    mut reset_events: MessageReader<ResetEvent>,
    rocket_dims: Res<RocketDimensions>,
    mut camera_properties: ResMut<CameraProperties>,
    virtual_time: Option<ResMut<Time<Virtual>>>,
    mut rocket_state: ResMut<RocketState>,
    mut locked_axes: Query<&mut LockedAxes, With<RocketMarker>>,
    mut rocket_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        (With<RocketMarker>, Without<Camera>),
    >,
    app_state: Res<State<AppState>>,
    mut rocket_cam_query: Query<(Entity, &mut Camera), (With<RocketCamMarker>, Without<DroneCamMarker>)>,
    mut drone_cam_query: Query<&mut Camera, (With<DroneCamMarker>, Without<RocketCamMarker>)>,
    cone_query: Query<Entity, With<RocketCone>>,
    mut scene_camera: ResMut<SceneCameraState>,
) {
    if reset_events.read().next().is_none() {
        return;
    }

    if let Ok((cam_entity, mut cam)) = rocket_cam_query.single_mut() {
        cam.is_active = false;
        if let Ok(cone_entity) = cone_query.single() {
            commands.entity(cone_entity).add_child(cam_entity);
            commands
                .entity(cam_entity)
                .insert(rocket_cam_mount_transform(&rocket_dims));
        }
    }
    if let Ok(mut cam) = drone_cam_query.single_mut() {
        cam.is_active = false;
    }
    camera_properties.aux_cam_enabled = false;
    camera_properties.camera_swapped = false;
    if let Some(mut time) = virtual_time {
        time.set_relative_speed(1.0);
    }
    rocket_state.state = RocketStateEnum::Initial;
    rocket_state.max_height = 0.0;
    rocket_state.max_velocity = 0.0;

    for mut axes in &mut locked_axes {
        *axes = lock_all_axes(LockedAxes::new());
    }

    let base_y = match *app_state.get() {
        AppState::Lab | AppState::Store => scene::TABLE_TOP_Y + rocket_dims.length * 0.5,
        _ => rocket_dims.length * 0.5,
    };

    if let Ok((rocket_ent, mut rocket_transform, mut lin_velocity, mut ang_velocity)) =
        rocket_query.single_mut()
    {
        rocket_transform.translation = Vec3::new(0.0, base_y, 0.0);
        rocket_transform.rotation = Quat::IDENTITY;
        *lin_velocity = LinearVelocity::ZERO;
        *ang_velocity = AngularVelocity::ZERO;
        rocket_state.launch_origin_y = rocket_transform.translation.y;
        commands.entity(rocket_ent).remove::<ForceTimer>();
    }

    scene_camera.clear(app_state.get());
    camera_properties.apply_scene_defaults(app_state.get());
}

fn detect_landing_from_collision_system(
    mut collision_events: MessageReader<CollisionStart>,
    rocket_query: Query<(Entity, &LinearVelocity), With<RocketMarker>>,
    mut rocket_state: ResMut<RocketState>,
    mut crash_events: MessageWriter<DownedEvent>,
) {
    if !matches!(
        rocket_state.state,
        RocketStateEnum::Launched | RocketStateEnum::Descending
    ) {
        return;
    }

    let Ok((rocket_entity, velocity)) = rocket_query.single() else {
        return;
    };
    if velocity.y > 0.25 {
        return;
    }

    let touched_ground = collision_events
        .read()
        .any(|event| event.body1 == Some(rocket_entity) || event.body2 == Some(rocket_entity));
    if touched_ground {
        info!("Rocket touchdown detected via collision event");
        rocket_state.state = RocketStateEnum::Grounded;
        crash_events.write(DownedEvent);
    }
}

fn wind_cardinal(dir: Vec2) -> &'static str {
    if dir.length_squared() < 0.001 {
        return "--";
    }
    let angle = dir.y.atan2(dir.x).to_degrees().rem_euclid(360.0);
    match angle {
        a if a < 22.5 => "E",
        a if a < 67.5 => "NE",
        a if a < 112.5 => "N",
        a if a < 157.5 => "NW",
        a if a < 202.5 => "W",
        a if a < 247.5 => "SW",
        a if a < 292.5 => "S",
        a if a < 337.5 => "SE",
        _ => "E",
    }
}

fn update_stats_system(
    rocket_state: Res<RocketState>,
    mut text_query: Query<&mut Text, With<ScoreMarker>>,
    rocket_query: Query<(&Transform, &LinearVelocity), (With<RocketMarker>, Without<Camera>)>,
) {
    let Ok(mut score_text) = text_query.single_mut() else {
        return;
    };
    let Ok((transform, velocity)) = rocket_query.single() else {
        return;
    };
    let altitude = (transform.translation.y - rocket_state.launch_origin_y).max(0.0);
    **score_text = format!(
        "Alt: {:5.1} / {:5.1} m\n\
         Vel: {:5.1} / {:5.1} m/s",
        altitude, rocket_state.max_height, velocity.length(), rocket_state.max_velocity
    );
}

fn update_wind_hud_system(
    sky_props: Res<SkyProperties>,
    wind: Res<wind::WindProperties>,
    mut icon_query: Query<(&mut Text, &mut TextColor), With<WindIconMarker>>,
    mut text_query: Query<&mut TextSpan, With<WindHudMarker>>,
) {
    let h = sky_props.time_of_day as u32 % 24;
    let m = ((sky_props.time_of_day.fract()) * 60.0) as u32;
    let is_day = (6..18).contains(&h);

    if let Ok((mut icon, mut icon_color)) = icon_query.single_mut() {
        **icon = if is_day { "* " } else { "( " }.into();
        icon_color.0 = if is_day {
            Color::srgb(1.0, 0.9, 0.2)
        } else {
            Color::srgb(0.6, 0.7, 1.0)
        };
    }

    if let Ok(mut text) = text_query.single_mut() {
        let horiz_speed =
            Vec2::new(wind.wind_velocity_world.x, wind.wind_velocity_world.z).length();
        let cardinal = wind_cardinal(wind.direction);
        **text = format!("{h:02}:{m:02}\nW: {horiz_speed:4.1} m/s {cardinal:<2}");
    }
}

#[derive(SystemParam)]
struct SaveUiParams<'w> {
    save_state: ResMut<'w, save::SaveState>,
    balance: ResMut<'w, save::PlayerBalance>,
    game_mode: ResMut<'w, save::GameMode>,
    owned_materials: ResMut<'w, save::OwnedMaterials>,
    rocket_cam_owned: ResMut<'w, save::RocketCamOwned>,
    parachute_config: ResMut<'w, parachute::ParachuteConfig>,
    inventory: ResMut<'w, inventory::Inventory>,
    owned_motor_sizes: ResMut<'w, inventory::OwnedMotorSizes>,
    owned_tube_types: ResMut<'w, inventory::OwnedTubeTypes>,
    owned_nosecone_types: ResMut<'w, inventory::OwnedNoseconeTypes>,
    equipped: ResMut<'w, inventory::EquippedLoadout>,
    experience: ResMut<'w, inventory::PlayerExperience>,
}

fn ui_system(
    mut contexts: EguiContexts,
    mut rocket_dims: ResMut<RocketDimensions>,
    mut rocket_flight_parameters: ResMut<RocketFlightParameters>,
    mut camera_properties: ResMut<CameraProperties>,
    mut sky_props: ResMut<SkyProperties>,
    mut sky_mode: ResMut<SkyRenderMode>,
    mut sun_disc_settings: ResMut<SunDiscSettings>,
    mut wind: ResMut<wind::WindProperties>,
    mut save_params: SaveUiParams,
    ambient_light: Res<GlobalAmbientLight>,
    sun_query: Query<&DirectionalLight, With<sky::SunLightMarker>>,
    rocket_query: Query<
        (&ComputedMass, &ComputedCenterOfMass),
        (With<RocketMarker>, Without<Camera>),
    >,
    mut fog_query: Query<&mut DistanceFog>,
    mut bloom_query: Query<&mut Bloom, (With<Camera3d>, Without<RocketCamMarker>, Without<DroneCamMarker>, Without<camera::EguiOverlayCam>)>,
    app_state: Res<State<AppState>>,
) -> Result {
    let save_state = &mut save_params.save_state;
    let balance = &mut save_params.balance;
    let game_mode = &mut save_params.game_mode;
    let owned_materials = &mut save_params.owned_materials;
    let rocket_cam_owned = &mut save_params.rocket_cam_owned;
    let parachute_config = &mut save_params.parachute_config;
    let inv = &mut save_params.inventory;
    let owned_motor_sizes = &mut save_params.owned_motor_sizes;
    let owned_tube_types = &mut save_params.owned_tube_types;
    let owned_nosecone_types = &mut save_params.owned_nosecone_types;
    let equipped = &mut save_params.equipped;
    let experience = &mut save_params.experience;
    let is_lab = *app_state.get() == AppState::Lab;
    let is_launch = *app_state.get() == AppState::Launch;
    let is_store = *app_state.get() == AppState::Store;
    let ctx = contexts.ctx_mut()?;
    camera_properties.egui_has_pointer = ctx.wants_pointer_input();

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut **game_mode, save::GameMode::Sandbox, "Sandbox");
                ui.selectable_value(&mut **game_mode, save::GameMode::Gameplay, "Gameplay");
            });
            if **game_mode == save::GameMode::Gameplay
                && !owned_materials.0.contains(&rocket_dims.material)
            {
                rocket_dims.material = RocketMaterial::Light;
                rocket_dims.flag_changed = true;
            }
            ui.add_space(4.0);

            egui::CollapsingHeader::new("Camera")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::Slider::new(&mut camera_properties.fixed_distance, 0.0..=50.0)
                            .text("distance"),
                    );
                    ui.add(
                        egui::Slider::new(&mut camera_properties.orbit_angle_degrees, 0.0..=360.0)
                            .text("orbit"),
                    );
                    ui.add(
                        egui::Slider::new(&mut camera_properties.desired_translation.y, 0.1..=20.0)
                            .text("elevation"),
                    );
                    ui.add(egui::Slider::new(&mut camera_properties.zoom, 0.5..=10.0).text("zoom"));
                    ui.add(
                        egui::Slider::new(&mut camera_properties.target_y_offset, -10.0..=10.0)
                            .text("look Y offset"),
                    );

                    let mode_label = match camera_properties.follow_mode {
                        FollowMode::FixedGround => "Ground",
                        FollowMode::FollowSide => "Side",
                        FollowMode::FollowAbove => "Above",
                        FollowMode::FreeLook => "Free Look",
                    };
                    egui::ComboBox::from_label("mode")
                        .selected_text(mode_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut camera_properties.follow_mode,
                                FollowMode::FixedGround,
                                "Ground",
                            );
                            ui.selectable_value(
                                &mut camera_properties.follow_mode,
                                FollowMode::FollowSide,
                                "Side",
                            );
                            ui.selectable_value(
                                &mut camera_properties.follow_mode,
                                FollowMode::FollowAbove,
                                "Above",
                            );
                            ui.selectable_value(
                                &mut camera_properties.follow_mode,
                                FollowMode::FreeLook,
                                "Free Look",
                            );
                        });

                    if is_launch {
                        let kind_label = match camera_properties.aux_cam_kind {
                            AuxCamKind::RocketCam => "Rocket Cam",
                            AuxCamKind::DroneCam => "Drone Cam",
                        };
                        let prev_kind = camera_properties.aux_cam_kind;
                        egui::ComboBox::from_label("aux cam")
                            .selected_text(kind_label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut camera_properties.aux_cam_kind,
                                    AuxCamKind::RocketCam,
                                    "Rocket Cam",
                                );
                                ui.selectable_value(
                                    &mut camera_properties.aux_cam_kind,
                                    AuxCamKind::DroneCam,
                                    "Drone Cam",
                                );
                            });
                        if camera_properties.aux_cam_kind != prev_kind {
                            camera_properties.camera_swapped = false;
                        }
                        if camera_properties.aux_cam_kind == AuxCamKind::DroneCam {
                            ui.add(
                                egui::Slider::new(&mut camera_properties.drone_sway, 0.0..=0.3)
                                    .text("drone sway"),
                            );
                            ui.horizontal(|ui| {
                                ui.label("alt");
                                ui.selectable_value(&mut camera_properties.drone_waypoint, DroneWaypoint::Ground, "5m");
                                ui.selectable_value(&mut camera_properties.drone_waypoint, DroneWaypoint::Low, "10m");
                                ui.selectable_value(&mut camera_properties.drone_waypoint, DroneWaypoint::High, "50m");
                                ui.selectable_value(&mut camera_properties.drone_waypoint, DroneWaypoint::Sky, "100m");
                            });
                            ui.horizontal(|ui| {
                                ui.label("dist");
                                ui.selectable_value(&mut camera_properties.drone_distance, DroneDistance::Near, "10m");
                                ui.selectable_value(&mut camera_properties.drone_distance, DroneDistance::Mid, "50m");
                                ui.selectable_value(&mut camera_properties.drone_distance, DroneDistance::Far, "100m");
                            });
                        }
                    }
                });

            if is_lab {
                ui.add_space(6.0);
                egui::CollapsingHeader::new("Rocket Body")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut changed = false;
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut rocket_dims.radius, 0.025..=0.5)
                                    .text("radius"),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut rocket_dims.length, 0.2..=2.0)
                                    .step_by(0.05)
                                    .text("body"),
                            )
                            .changed();
                        let cone_max = rocket_dims.length * 0.4;
                        rocket_dims.cone_length = rocket_dims.cone_length.min(cone_max);
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut rocket_dims.cone_length, 0.01..=cone_max)
                                    .text("cone"),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut rocket_dims.num_fins, 1.0..=8.0)
                                    .step_by(1.0)
                                    .text("fins"),
                            )
                            .changed();
                        let fin_h_max = rocket_dims.length * 0.8;
                        rocket_dims.fin_height = rocket_dims.fin_height.min(fin_h_max);
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut rocket_dims.fin_height, 0.01..=fin_h_max)
                                    .step_by(0.1)
                                    .text("fin H"),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut rocket_dims.fin_length, 0.01..=1.0)
                                    .step_by(0.1)
                                    .text("fin L"),
                            )
                            .changed();
                        if changed {
                            rocket_dims.flag_changed = true;
                        }

                        ui.separator();
                        let mut color_changed = false;
                        let color_combo =
                            |ui: &mut egui::Ui, label: &str, current: &mut ColorPreset| {
                                let prev = *current;
                                egui::ComboBox::from_label(label)
                                    .selected_text(current.label())
                                    .show_ui(ui, |ui| {
                                        for preset in ColorPreset::ALL {
                                            ui.selectable_value(current, preset, preset.label());
                                        }
                                    });
                                *current != prev
                            };
                        color_changed |= color_combo(ui, "body", &mut rocket_dims.body_color);
                        color_changed |= color_combo(ui, "cone", &mut rocket_dims.cone_color);
                        color_changed |= color_combo(ui, "fins", &mut rocket_dims.fin_color);
                        if color_changed {
                            rocket_dims.flag_changed = true;
                        }

                        ui.separator();
                        let prev_mat = rocket_dims.material;
                        let available_materials: Vec<RocketMaterial> =
                            if **game_mode == save::GameMode::Gameplay {
                                RocketMaterial::ALL
                                    .iter()
                                    .copied()
                                    .filter(|m| owned_materials.0.contains(m))
                                    .collect()
                            } else {
                                RocketMaterial::ALL.to_vec()
                            };
                        egui::ComboBox::from_label("material")
                            .selected_text(rocket_dims.material.label())
                            .show_ui(ui, |ui| {
                                for preset in &available_materials {
                                    ui.selectable_value(
                                        &mut rocket_dims.material,
                                        *preset,
                                        preset.label(),
                                    );
                                }
                            });
                        if rocket_dims.material != prev_mat {
                            rocket_dims.flag_changed = true;
                        }

                        if let Ok((mass, com)) = rocket_query.single() {
                            ui.separator();
                            ui.label(format!(
                                "Mass: {:.3}  CoM: ({:.2}, {:.2}, {:.2})",
                                mass.value(),
                                com.0.x,
                                com.0.y,
                                com.0.z
                            ));
                        }
                    });

                ui.add_space(6.0);
                egui::CollapsingHeader::new("Engine")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Slider::new(&mut rocket_flight_parameters.force, 0.5..=10.0)
                                .step_by(0.01)
                                .text("force"),
                        );
                        ui.add(
                            egui::Slider::new(&mut rocket_flight_parameters.duration, 0.5..=10.0)
                                .text("duration"),
                        );
                    });

                ui.add_space(6.0);
                egui::CollapsingHeader::new("Parachute")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Slider::new(&mut parachute_config.diameter, 0.1..=1.0)
                                .step_by(0.01)
                                .text("diameter (m)"),
                        );
                        ui.add(
                            egui::Slider::new(&mut parachute_config.deploy_delay, 0.5..=10.0)
                                .step_by(0.1)
                                .text("deploy delay (s)"),
                        );
                    });

                if **game_mode == save::GameMode::Gameplay {
                    ui.add_space(6.0);
                    egui::CollapsingHeader::new("Loadout")
                        .default_open(true)
                        .show(ui, |ui| {
                            let prev_motor = equipped.motor;
                            egui::ComboBox::from_label("motor")
                                .selected_text(equipped.motor.label())
                                .show_ui(ui, |ui| {
                                    for size in inventory::MotorSize::ALL {
                                        if owned_motor_sizes.0.contains(&size)
                                            && inv.motor_count(size) > 0
                                        {
                                            let count = inv.motor_count(size);
                                            ui.selectable_value(
                                                &mut equipped.motor,
                                                size,
                                                format!("{} ({})", size.label(), count),
                                            );
                                        }
                                    }
                                });
                            if equipped.motor != prev_motor {
                                info!("Equipped motor: {}", equipped.motor.label());
                            }

                            let prev_chute = equipped.parachute;
                            egui::ComboBox::from_label("parachute")
                                .selected_text(equipped.parachute.label())
                                .show_ui(ui, |ui| {
                                    for size in inventory::ParachuteSize::ALL {
                                        if inv.parachute_count(size) > 0 {
                                            ui.selectable_value(
                                                &mut equipped.parachute,
                                                size,
                                                size.label(),
                                            );
                                        }
                                    }
                                });
                            if equipped.parachute != prev_chute {
                                info!("Equipped parachute: {}", equipped.parachute.label());
                            }

                            let prev_tube = equipped.tube_type;
                            egui::ComboBox::from_label("tube")
                                .selected_text(equipped.tube_type.label())
                                .show_ui(ui, |ui| {
                                    for tt in inventory::TubeType::ALL {
                                        if owned_tube_types.0.contains(&tt) {
                                            ui.selectable_value(
                                                &mut equipped.tube_type,
                                                tt,
                                                tt.label(),
                                            );
                                        }
                                    }
                                });
                            if equipped.tube_type != prev_tube {
                                info!("Equipped tube: {}", equipped.tube_type.label());
                            }

                            let prev_nose = equipped.nosecone_type;
                            egui::ComboBox::from_label("nosecone")
                                .selected_text(equipped.nosecone_type.label())
                                .show_ui(ui, |ui| {
                                    for nt in inventory::NoseconeType::ALL {
                                        if owned_nosecone_types.0.contains(&nt) {
                                            ui.selectable_value(
                                                &mut equipped.nosecone_type,
                                                nt,
                                                nt.label(),
                                            );
                                        }
                                    }
                                });
                            if equipped.nosecone_type != prev_nose {
                                info!("Equipped nosecone: {}", equipped.nosecone_type.label());
                            }
                        });
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.add_space(6.0);
                    egui::CollapsingHeader::new("Rocket Saves")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label(format!("Player: {}", save_state.player_name));
                            ui.separator();

                            {
                                let player = save_state.player_name.clone();
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(&mut save_state.rocket_name_buf);
                                    if ui.button("Save").clicked()
                                        && !save_state.rocket_name_buf.trim().is_empty()
                                    {
                                        let rname = save_state.rocket_name_buf.trim().to_string();
                                        match save::save_rocket(
                                            &player,
                                            &rname,
                                            &rocket_dims,
                                            &rocket_flight_parameters,
                                        ) {
                                            Ok(()) => {
                                                save_state.rocket_saves =
                                                    save::list_rockets(&player);
                                                save_state.status_message =
                                                    Some(format!("Saved '{rname}'"));
                                            }
                                            Err(e) => {
                                                save_state.status_message = Some(e);
                                            }
                                        }
                                    }
                                });

                                ui.separator();
                                let mut action: Option<(String, bool)> = None;
                                for rocket_name in &save_state.rocket_saves {
                                    ui.horizontal(|ui| {
                                        ui.label(rocket_name);
                                        if ui.small_button("Load").clicked() {
                                            action = Some((rocket_name.clone(), false));
                                        }
                                        if ui.small_button("Del").clicked() {
                                            action = Some((rocket_name.clone(), true));
                                        }
                                    });
                                }
                                if let Some((rname, is_delete)) = action {
                                    if is_delete {
                                        match save::delete_rocket(&player, &rname) {
                                            Ok(()) => {
                                                save_state.rocket_saves =
                                                    save::list_rockets(&player);
                                                save_state.status_message =
                                                    Some(format!("Deleted '{rname}'"));
                                            }
                                            Err(e) => {
                                                save_state.status_message = Some(e);
                                            }
                                        }
                                    } else {
                                        match save::load_rocket(&player, &rname) {
                                            Ok(data) => {
                                                *rocket_dims = data.dimensions;
                                                rocket_dims.flag_changed = true;
                                                *rocket_flight_parameters = data.flight_params;
                                                save_state.rocket_name_buf = rname.clone();
                                                save_state.status_message =
                                                    Some(format!("Loaded '{rname}'"));
                                            }
                                            Err(e) => {
                                                save_state.status_message = Some(e);
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(msg) = &save_state.status_message {
                                ui.separator();
                                ui.label(msg);
                            }
                        });
                }
            }

            if is_launch {
                ui.add_space(6.0);
                egui::CollapsingHeader::new("Sky")
                    .default_open(false)
                    .show(ui, |ui| {
                        let is_cubemap_mode = *sky_mode == SkyRenderMode::Cubemap;
                        let mode_label = match *sky_mode {
                            SkyRenderMode::Cubemap => "Cubemap",
                            SkyRenderMode::Atmosphere => "Atmosphere",
                        };
                        egui::ComboBox::from_label("sky mode")
                            .selected_text(mode_label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut *sky_mode,
                                    SkyRenderMode::Cubemap,
                                    "Cubemap",
                                );
                                // Atmosphere mode crashes with bevy_firework's render pipeline
                                // (bind group layout mismatch, bevyengine/bevy#21784)
                                ui.disable();
                                ui.selectable_value(
                                    &mut *sky_mode,
                                    SkyRenderMode::Atmosphere,
                                    "Atmosphere (needs particle fix)",
                                );
                            });

                        ui.separator();
                        ui.add(
                            egui::Slider::new(&mut sky_props.time_of_day, 0.0..=24.0).text("time"),
                        );
                        ui.add(
                            egui::Slider::new(&mut sky_props.day_speed, 0.0..=600.0).text("speed"),
                        );
                        ui.add(
                            egui::Slider::new(&mut sky_props.ambient_floor, 0.0..=1.0)
                                .text("ambient floor"),
                        );
                        ui.label(format!(
                            "  ambient brightness: {:.3}",
                            ambient_light.brightness
                        ));
                        if let Ok(sun) = sun_query.single() {
                            ui.label(format!("  sun illuminance: {:.1} lux", sun.illuminance));
                        }

                        ui.separator();
                        if is_cubemap_mode {
                            let current_name = SKYBOXES[sky_props.skybox_index].name;
                            let mut changed = false;
                            egui::ComboBox::from_label("skybox")
                                .selected_text(current_name)
                                .show_ui(ui, |ui| {
                                    for (i, entry) in SKYBOXES.iter().enumerate() {
                                        if ui
                                            .selectable_value(
                                                &mut sky_props.skybox_index,
                                                i,
                                                entry.name,
                                            )
                                            .changed()
                                        {
                                            changed = true;
                                        }
                                    }
                                });
                            if changed {
                                sky_props.lab_skybox_index = sky_props.skybox_index;
                                sky_props.skybox_changed = true;
                                if sky_props.fog_enabled
                                    && let Ok(mut fog_settings) = fog_query.single_mut()
                                {
                                    apply_fog_mode(
                                        &mut fog_settings,
                                        sky_props.fog_mode,
                                        sky_props.fog_visibility,
                                        sky_props.skybox_index,
                                    );
                                }
                            }

                            ui.separator();
                            ui.checkbox(&mut sun_disc_settings.enabled, "Sun disc");
                            ui.add(
                                egui::Slider::new(
                                    &mut sun_disc_settings.emissive_strength,
                                    1000.0..=80000.0,
                                )
                                .logarithmic(true)
                                .text("sun emissive"),
                            );
                            ui.add(
                                egui::Slider::new(
                                    &mut sun_disc_settings.angular_diameter_deg,
                                    0.1..=1.5,
                                )
                                .text("sun size (deg)"),
                            );
                            ui.separator();
                        } else {
                            ui.label("Cubemap-only controls are hidden in Atmosphere mode.");
                            ui.separator();
                        }

                        if let Ok(mut bloom) = bloom_query.single_mut() {
                            ui.add(
                                egui::Slider::new(&mut bloom.intensity, 0.0..=1.0).text("bloom"),
                            );
                            ui.add(
                                egui::Slider::new(&mut bloom.high_pass_frequency, 0.0..=1.0)
                                    .text("bloom high-pass"),
                            );
                        }

                        ui.separator();
                        ui.checkbox(
                            &mut sky_props.volumetrics_enabled,
                            "Volumetrics (opt-in, GPU heavy)",
                        );

                        ui.separator();
                        let mut fog_changed = false;
                        let fog_label = match sky_props.fog_mode {
                            1 => "Atmospheric",
                            2 => "Dense",
                            _ => "Off",
                        };
                        egui::ComboBox::from_label("Fog")
                            .selected_text(fog_label)
                            .show_ui(ui, |ui| {
                                for (m, label) in [(0, "Off"), (1, "Atmospheric"), (2, "Dense")] {
                                    if ui
                                        .selectable_value(&mut sky_props.fog_mode, m, label)
                                        .changed()
                                    {
                                        sky_props.fog_enabled = m > 0;
                                        fog_changed = true;
                                    }
                                }
                            });

                        if sky_props.fog_enabled
                            && ui
                                .add(
                                    egui::Slider::new(&mut sky_props.fog_visibility, 10.0..=200.0)
                                        .text("visibility"),
                                )
                                .changed()
                        {
                            fog_changed = true;
                        }

                        if fog_changed && let Ok(mut fog_settings) = fog_query.single_mut() {
                            apply_fog_mode(
                                &mut fog_settings,
                                sky_props.fog_mode,
                                sky_props.fog_visibility,
                                sky_props.skybox_index,
                            );
                        }
                    });
            }

            if is_launch {
                ui.add_space(6.0);
                egui::CollapsingHeader::new("Wind")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(&mut wind.strength, 0.0..=1.0).text("strength"));
                    });
            }

            #[cfg(not(target_arch = "wasm32"))]
            if is_store {
                ui.add_space(6.0);
                egui::CollapsingHeader::new("Shop")
                    .default_open(true)
                    .show(ui, |ui| {
                        if **game_mode == save::GameMode::Sandbox {
                            ui.label("Sandbox mode — everything unlocked.");
                            return;
                        }

                        let player = save_state.player_name.clone();
                        if let Err(e) = save::ensure_player_dir(&player) {
                            save_state.status_message = Some(format!("Error: {e}"));
                            return;
                        }

                        let save_meta = |balance: &save::PlayerBalance,
                                         owned_materials: &save::OwnedMaterials,
                                         rocket_cam_owned: &save::RocketCamOwned,
                                         inv: &inventory::Inventory,
                                         owned_motor_sizes: &inventory::OwnedMotorSizes,
                                         owned_tube_types: &inventory::OwnedTubeTypes,
                                         owned_nosecone_types: &inventory::OwnedNoseconeTypes,
                                         experience: &inventory::PlayerExperience,
                                         player: &str| {
                            let meta = save::build_player_meta(
                                player,
                                balance.0,
                                &owned_materials.0,
                                rocket_cam_owned.0,
                                inv,
                                &owned_motor_sizes.0,
                                &owned_tube_types.0,
                                &owned_nosecone_types.0,
                                experience.0,
                            );
                            let _ = save::save_player_meta(&meta);
                        };

                        // Starter rocket purchase
                        let owns_starter = save_state.rocket_saves.contains(&"Starter".to_string());
                        if owns_starter {
                            ui.label("Starter Rocket — Purchased!");
                        } else {
                            let can_afford = balance.0 >= 10.0;
                            let btn = egui::Button::new("Buy Starter Rocket — $10");
                            if ui.add_enabled(can_afford, btn).clicked() {
                                let starter_dims = RocketDimensions {
                                    radius: 0.02,
                                    length: 0.35,
                                    cone_length: 0.08,
                                    num_fins: 3.0,
                                    fin_height: 0.15,
                                    fin_length: 0.08,
                                    body_color: ColorPreset::White,
                                    cone_color: ColorPreset::White,
                                    fin_color: ColorPreset::White,
                                    material: RocketMaterial::Light,
                                    flag_changed: false,
                                };
                                let default_params = RocketFlightParameters::default();
                                if let Err(e) = save::save_rocket(&player, "Starter", &starter_dims, &default_params) {
                                    save_state.status_message = Some(format!("Error: {e}"));
                                } else {
                                    balance.0 -= 10.0;
                                    save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                    save_state.rocket_saves = save::list_rockets(&player);
                                    save_state.status_message = Some("Purchased Starter Rocket!".to_string());
                                }
                            }
                            if !can_afford {
                                ui.label("Not enough funds.");
                            }
                        }

                        ui.separator();
                        ui.label("Materials");
                        for mat in RocketMaterial::ALL {
                            if mat.price() == 0.0 {
                                continue;
                            }
                            if owned_materials.0.contains(&mat) {
                                ui.label(format!("{} — Owned", mat.label()));
                            } else {
                                let price = mat.price();
                                let can_afford = balance.0 >= price;
                                let btn = egui::Button::new(format!("Buy {} — ${}", mat.label(), price));
                                if ui.add_enabled(can_afford, btn).clicked() {
                                    balance.0 -= price;
                                    owned_materials.0.push(mat);
                                    save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                    save_state.status_message = Some(format!("Purchased {}!", mat.label()));
                                }
                                if !can_afford {
                                    ui.label("Not enough funds.");
                                }
                            }
                        }

                        ui.separator();
                        ui.label("Upgrades");
                        if rocket_cam_owned.0 {
                            ui.label("Rocket Cam — Owned");
                        } else {
                            let price = 15.0;
                            let can_afford = balance.0 >= price;
                            let btn = egui::Button::new(format!("Buy Rocket Cam — ${price}"));
                            if ui.add_enabled(can_afford, btn).clicked() {
                                balance.0 -= price;
                                rocket_cam_owned.0 = true;
                                save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                save_state.status_message = Some("Purchased Rocket Cam!".to_string());
                            }
                            if !can_afford {
                                ui.label("Not enough funds.");
                            }
                        }

                        ui.separator();
                        ui.label("Motor Unlocks");
                        for size in inventory::MotorSize::ALL {
                            let unlock_price = size.unlock_price();
                            if unlock_price == 0.0 {
                                continue;
                            }
                            if owned_motor_sizes.0.contains(&size) {
                                ui.label(format!("{} Motors — Unlocked", size.label()));
                            } else {
                                let can_afford = balance.0 >= unlock_price;
                                let btn = egui::Button::new(format!(
                                    "Unlock {} Motors — ${unlock_price}",
                                    size.label()
                                ));
                                if ui.add_enabled(can_afford, btn).clicked() {
                                    balance.0 -= unlock_price;
                                    owned_motor_sizes.0.push(size);
                                    save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                    save_state.status_message =
                                        Some(format!("Unlocked {} Motors!", size.label()));
                                }
                                if !can_afford {
                                    ui.label("Not enough funds.");
                                }
                            }
                        }

                        ui.separator();
                        ui.label("Buy Motors");
                        for size in inventory::MotorSize::ALL {
                            if !owned_motor_sizes.0.contains(&size) {
                                continue;
                            }
                            let pack_price = size.pack_price();
                            let count = inv.motor_count(size);
                            let can_afford = balance.0 >= pack_price;
                            let btn = egui::Button::new(format!(
                                "Buy 3x {} Motors — ${pack_price}  (have {count})",
                                size.label()
                            ));
                            if ui.add_enabled(can_afford, btn).clicked() {
                                balance.0 -= pack_price;
                                inv.add_motors(size, 3);
                                save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                save_state.status_message =
                                    Some(format!("Purchased 3x {} Motors!", size.label()));
                            }
                            if !can_afford {
                                ui.label("Not enough funds.");
                            }
                        }

                        ui.separator();
                        ui.label("Parachutes");
                        for size in inventory::ParachuteSize::ALL {
                            let count = inv.parachute_count(size);
                            if count > 0 {
                                ui.label(format!("{} Parachute — Owned", size.label()));
                            } else {
                                let price = size.price();
                                let can_afford = balance.0 >= price;
                                let btn = egui::Button::new(format!(
                                    "Buy {} Parachute — ${price}",
                                    size.label()
                                ));
                                if ui.add_enabled(can_afford, btn).clicked() {
                                    balance.0 -= price;
                                    *inv.parachutes.entry(size).or_insert(0) += 1;
                                    save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                    save_state.status_message =
                                        Some(format!("Purchased {} Parachute!", size.label()));
                                }
                                if !can_afford {
                                    ui.label("Not enough funds.");
                                }
                            }
                        }

                        ui.separator();
                        ui.label("Tube Types");
                        for tt in inventory::TubeType::ALL {
                            if tt.price() == 0.0 {
                                continue;
                            }
                            if owned_tube_types.0.contains(&tt) {
                                ui.label(format!("{} — Owned", tt.label()));
                            } else {
                                let price = tt.price();
                                let can_afford = balance.0 >= price;
                                let btn = egui::Button::new(format!(
                                    "Buy {} Tube — ${price}",
                                    tt.label()
                                ));
                                if ui.add_enabled(can_afford, btn).clicked() {
                                    balance.0 -= price;
                                    owned_tube_types.0.push(tt);
                                    save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                    save_state.status_message =
                                        Some(format!("Purchased {} Tube!", tt.label()));
                                }
                                if !can_afford {
                                    ui.label("Not enough funds.");
                                }
                            }
                        }

                        ui.separator();
                        ui.label("Nosecone Types");
                        for nt in inventory::NoseconeType::ALL {
                            if nt.price() == 0.0 {
                                continue;
                            }
                            if owned_nosecone_types.0.contains(&nt) {
                                ui.label(format!("{} — Owned", nt.label()));
                            } else {
                                let price = nt.price();
                                let can_afford = balance.0 >= price;
                                let btn = egui::Button::new(format!(
                                    "Buy {} Nosecone — ${price}",
                                    nt.label()
                                ));
                                if ui.add_enabled(can_afford, btn).clicked() {
                                    balance.0 -= price;
                                    owned_nosecone_types.0.push(nt);
                                    save_meta(balance, owned_materials, rocket_cam_owned, inv, owned_motor_sizes, owned_tube_types, owned_nosecone_types, experience, &player);
                                    save_state.status_message =
                                        Some(format!("Purchased {} Nosecone!", nt.label()));
                                }
                                if !can_afford {
                                    ui.label("Not enough funds.");
                                }
                            }
                        }
                    });
            }

            ui.add_space(6.0);
            egui::CollapsingHeader::new("Keys")
                .default_open(false)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for line in [
                        "Tab: cycle Lab/Launch/Store",
                        "WASD: move (FreeLook)",
                        "Hold `/~: slow motion",
                        "Arrows: orbit / distance",
                        "Shift+Up/Down: truck cam",
                        "Esc: world inspector",
                        "F10: collider gizmos",
                        "F11: profiling HUD",
                        "V: aux-cam PiP / Shift+V: swap",
                        "F12: toggle FPS",
                    ] {
                        ui.label(line);
                    }
                });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    Ok(())
}

#[derive(Resource, Default)]
struct CameraDebugOpen(bool);

fn camera_debug_system(
    mut contexts: EguiContexts,
    mut debug_open: ResMut<CameraDebugOpen>,
    camera_properties: Res<CameraProperties>,
    main_cam_query: Query<&Transform, (With<Camera3d>, Without<RocketCamMarker>, Without<DroneCamMarker>, Without<camera::EguiOverlayCam>)>,
    rocket_cam_query: Query<(&Camera, &GlobalTransform), (With<RocketCamMarker>, Without<DroneCamMarker>)>,
    drone_cam_query: Query<(&Camera, &GlobalTransform), (With<DroneCamMarker>, Without<RocketCamMarker>)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    if ctx.input(|i| i.key_pressed(Key::F9)) {
        debug_open.0 = !debug_open.0;
    }
    if !debug_open.0 {
        return Ok(());
    }

    egui::Window::new("Camera Debug")
        .default_open(true)
        .default_pos([300.0, 10.0])
        .resizable(false)
        .show(ctx, |ui| {
            let fmt_v3 = |v: Vec3| format!("({:6.2}, {:6.2}, {:6.2})", v.x, v.y, v.z);
            let mode_str = match camera_properties.follow_mode {
                FollowMode::FreeLook => "FreeLook",
                FollowMode::FixedGround => "FixedGround",
                FollowMode::FollowSide => "FollowSide",
                FollowMode::FollowAbove => "FollowAbove",
            };

            ui.label(egui::RichText::new("Main Camera").strong());
            if let Ok(tf) = main_cam_query.single() {
                ui.monospace(format!("pos:    {}", fmt_v3(tf.translation)));
                let fwd = tf.forward().as_vec3();
                ui.monospace(format!("fwd:    {}", fmt_v3(fwd)));
            }
            ui.monospace(format!("target: {}", fmt_v3(camera_properties.target)));
            ui.monospace(format!("mode:   {mode_str}"));
            ui.monospace(format!("zoom:   {:.1}x", camera_properties.zoom));
            ui.monospace(format!("orbit:  {:.1}°", camera_properties.orbit_angle_degrees));
            ui.monospace(format!("dist:   {:.1}", camera_properties.fixed_distance));
            ui.separator();

            if let Ok((cam, gtf)) = rocket_cam_query.single() {
                ui.label(egui::RichText::new("Rocket Cam").strong());
                let status = if cam.is_active { "ACTIVE" } else { "off" };
                ui.monospace(format!("status: {status}"));
                let pos = gtf.translation();
                let fwd = gtf.forward().as_vec3();
                ui.monospace(format!("pos:    {}", fmt_v3(pos)));
                ui.monospace(format!("fwd:    {}", fmt_v3(fwd)));
                ui.separator();
            }

            if let Ok((cam, gtf)) = drone_cam_query.single() {
                ui.label(egui::RichText::new("Drone Cam").strong());
                let status = if cam.is_active { "ACTIVE" } else { "off" };
                ui.monospace(format!("status: {status}"));
                let pos = gtf.translation();
                let fwd = gtf.forward().as_vec3();
                ui.monospace(format!("pos:    {}", fmt_v3(pos)));
                ui.monospace(format!("fwd:    {}", fmt_v3(fwd)));
            }
        });

    Ok(())
}

fn sync_equipped_loadout(
    equipped: Res<inventory::EquippedLoadout>,
    game_mode: Res<save::GameMode>,
    mut parachute_config: ResMut<parachute::ParachuteConfig>,
) {
    if *game_mode != save::GameMode::Gameplay || !equipped.is_changed() {
        return;
    }
    parachute_config.diameter = equipped.parachute.diameter();
}

fn update_rocket_dimensions_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rocket_dims: ResMut<RocketDimensions>,
    mut geometry_changed: MessageWriter<RocketGeometryChangedEvent>,
    mut body_query: Query<
        (
            &mut Mesh3d,
            &mut Collider,
            &mut Transform,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (With<RocketBody>, Without<RocketCone>),
    >,
    mut cone_query: Query<
        (
            &mut Mesh3d,
            &mut Collider,
            &mut Transform,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (With<RocketCone>, Without<RocketBody>),
    >,
    mut rb_query: Query<
        (
            &mut Transform,
            &mut Mass,
            &mut AngularInertia,
            &mut CenterOfMass,
        ),
        (With<RocketMarker>, Without<RocketCone>, Without<RocketBody>),
    >,
    rocket_query: Query<Entity, With<RocketMarker>>,
    mut fins_query: Query<Entity, With<FinMarker>>,
    mut mass_model: ResMut<RocketMassModel>,
    app_state: Res<State<AppState>>,
) {
    if !rocket_dims.flag_changed {
        return;
    }

    debug!("Updating rocket dimensions");
    *mass_model = rocket_dims.material.to_mass_model();

    let base_y = match *app_state.get() {
        AppState::Lab | AppState::Store => scene::TABLE_TOP_Y + rocket_dims.length * 0.5,
        _ => rocket_dims.length * 0.5,
    };
    let mass_properties = rocket_mass_properties(rocket_dims.as_ref(), mass_model.as_ref());
    for (mut rb_transform, mut mass, mut inertia, mut center_of_mass) in rb_query.iter_mut() {
        rb_transform.translation.y = base_y;
        *mass = mass_properties.mass;
        *inertia = mass_properties.angular_inertia;
        *center_of_mass = mass_properties.center_of_mass;
    }

    let Ok(rocket) = rocket_query.single() else {
        warn!("Rocket dimension update requested but no rocket entity exists");
        return;
    };

    for (mut mesh_handle, mut collider, _, mat_handle) in body_query.iter_mut() {
        *mesh_handle = Mesh3d(
            meshes.add(
                Cylinder::new(rocket_dims.radius, rocket_dims.length)
                    .mesh()
                    .resolution(rocket::CIRCLE_RESOLUTION),
            ),
        );
        *collider = Collider::cylinder(rocket_dims.radius, rocket_dims.length);
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = rocket_dims.body_color.to_color();
        }
    }

    for (mut mesh_handle, mut collider, mut transform, mat_handle) in cone_query.iter_mut() {
        *mesh_handle = Mesh3d(meshes.add(Mesh::from(Cone {
            radius: rocket_dims.radius,
            height: rocket_dims.cone_length,
            segments: rocket::CIRCLE_RESOLUTION,
        })));
        *collider = Collider::cone(rocket_dims.radius, rocket_dims.cone_length);
        transform.translation.y = rocket_dims.total_length() * 0.5;
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = rocket_dims.cone_color.to_color();
        }
    }

    // Remove fins
    for fin in fins_query.iter_mut() {
        debug!("Removing fins");
        commands.entity(fin).despawn();
    }
    // Add fins
    let rocket_fin_pbr_bundles = create_rocket_fin_pbr_bundles(
        materials.as_mut(),
        rocket_dims.as_ref(),
        meshes.as_mut(),
        rocket_dims.fin_color.to_color(),
    );
    for bundle in rocket_fin_pbr_bundles {
        commands.entity(rocket).with_children(|parent| {
            parent.spawn((bundle, FinMarker));
        });
    }
    rocket_dims.flag_changed = false;
    geometry_changed.write_default();
}

// NOTE: Track filenames are misleading — "Rocket_Town" is the lab track,
// "the_Lab" is the outdoor launch track (legacy naming).
fn spawn_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/Welcome_to_Rocket_Town_v1.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(0.0),
            ..default()
        },
        LabMusicMarker,
    ));
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/Welcome_to_the_Lab_v1.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(0.0),
            ..default()
        },
        LaunchMusicMarker,
    ));
}

const MUSIC_FADE_SPEED: f32 = 1.5;

fn music_crossfade_system(
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    audio_settings: Res<AudioSettings>,
    mut lab_query: Query<&mut AudioSink, (With<LabMusicMarker>, Without<LaunchMusicMarker>)>,
    mut launch_query: Query<&mut AudioSink, (With<LaunchMusicMarker>, Without<LabMusicMarker>)>,
) {
    let master = if audio_settings.music_enabled {
        1.0_f32
    } else {
        0.0
    };
    let (lab_target, launch_target) = match app_state.get() {
        AppState::Lab => (master, 0.0),
        AppState::Launch => (0.0, master),
        _ => (0.0, 0.0),
    };

    let dt = time.delta_secs() * MUSIC_FADE_SPEED;

    if let Ok(mut sink) = lab_query.single_mut() {
        let current = sink.volume().to_linear();
        let new = move_toward(current, lab_target, dt);
        sink.set_volume(bevy::audio::Volume::Linear(new));
    }
    if let Ok(mut sink) = launch_query.single_mut() {
        let current = sink.volume().to_linear();
        let new = move_toward(current, launch_target, dt);
        sink.set_volume(bevy::audio::Volume::Linear(new));
    }
}

fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

const DEFAULT_FOV_DEGREES: f32 = 45.0;

fn spawn_drone_cam_system(
    mut commands: Commands,
    main_cam_query: Query<&Skybox, (With<Camera3d>, Without<RocketCone>, Without<camera::EguiOverlayCam>)>,
) {
    let Ok(skybox) = main_cam_query.single() else {
        return;
    };
    let skybox = skybox.clone();

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            is_active: false,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Hdr,
        Tonemapping::TonyMcMapface,
        Projection::Perspective(PerspectiveProjection {
            fov: DRONE_CAM_FOV_DEGREES.to_radians(),
            ..default()
        }),
        Transform::from_translation(DRONE_CAM_POSITION)
            .looking_at(DRONE_CAM_POSITION + Vec3::NEG_Z, Vec3::Y),
        DroneCamMarker,
        skybox,
        Bloom {
            intensity: 0.15,
            ..Bloom::NATURAL
        },
        Name::new("DroneCam"),
    ));
}

const DRONE_SPRING_STIFFNESS: f32 = 2.0;
const DRONE_SPRING_DAMPING: f32 = 3.0;

const DRONE_ALT_GROUND: f32 = 5.0;
const DRONE_ALT_LOW: f32 = 10.0;
const DRONE_ALT_HIGH: f32 = 50.0;
const DRONE_ALT_SKY: f32 = 100.0;

const DRONE_DIST_NEAR: f32 = 10.0;
const DRONE_DIST_MID: f32 = 50.0;
const DRONE_DIST_FAR: f32 = 100.0;

#[derive(Default)]
struct DroneMotionState {
    angular_velocity: Vec3,
    gust_timer: f32,
    target_rotation: Option<Quat>,
    position_velocity: Vec3,
}

fn drone_cam_track_rocket_system(
    time: Res<Time>,
    camera_properties: Res<CameraProperties>,
    mut state: Local<DroneMotionState>,
    mut query: Query<(&Camera, &mut Transform), With<DroneCamMarker>>,
) {
    let Ok((cam, mut tf)) = query.single_mut() else { return };
    if !cam.is_active || camera_properties.aux_cam_kind != AuxCamKind::DroneCam {
        state.target_rotation = None;
        return;
    }
    let dt = time.delta_secs();
    if dt == 0.0 { return; }

    let target = *state.target_rotation.get_or_insert(tf.rotation);

    // Spring-damper rotation
    let error_quat = target * tf.rotation.inverse();
    let (axis, angle) = error_quat.to_axis_angle();
    let error = if angle.abs() > std::f32::consts::PI {
        axis * (angle - std::f32::consts::TAU * angle.signum())
    } else {
        axis * angle
    };
    let torque = error * DRONE_SPRING_STIFFNESS - state.angular_velocity * DRONE_SPRING_DAMPING;
    state.angular_velocity += torque * dt;
    tf.rotation *= Quat::from_scaled_axis(state.angular_velocity * dt);

    // Wind gust impulses
    state.gust_timer -= dt;
    if state.gust_timer <= 0.0 {
        let t = time.elapsed_secs();
        let sway = camera_properties.drone_sway;
        let yaw_impulse = (t * 7.31).sin() * 0.04 * sway;
        let pitch_impulse = (t * 13.17).sin() * 0.015 * sway;
        state.angular_velocity += Vec3::new(pitch_impulse, yaw_impulse, 0.0);
        state.gust_timer = 2.0 + ((t * 3.73).sin() + 1.0) * 1.5;
    }

    // Spring position toward selected waypoint
    let alt = match camera_properties.drone_waypoint {
        DroneWaypoint::Ground => DRONE_ALT_GROUND,
        DroneWaypoint::Low => DRONE_ALT_LOW,
        DroneWaypoint::High => DRONE_ALT_HIGH,
        DroneWaypoint::Sky => DRONE_ALT_SKY,
    };
    let dist = match camera_properties.drone_distance {
        DroneDistance::Near => DRONE_DIST_NEAR,
        DroneDistance::Mid => DRONE_DIST_MID,
        DroneDistance::Far => DRONE_DIST_FAR,
    };
    let waypoint_target = Vec3::new(0.0, alt, dist);
    spring_to_target(
        &mut tf.translation,
        &mut state.position_velocity,
        waypoint_target,
        1.5,
        1.0,
        30.0,
        dt,
    );

    // Recompute target rotation from current position so it faces the horizon
    let horizon_target = tf.translation + Vec3::NEG_Z;
    let desired_rot = Transform::from_translation(tf.translation)
        .looking_at(horizon_target, Vec3::Y)
        .rotation;
    state.target_rotation = Some(desired_rot);

    // Vertical bob on top of spring-interpolated position
    let bob = 0.15 * camera_properties.drone_sway * (time.elapsed_secs() * 0.7 * std::f32::consts::TAU).sin();
    tf.translation.y += bob;
}

fn sync_aux_cam_kind_system(
    camera_properties: Res<CameraProperties>,
    mut prev_kind: Local<AuxCamKind>,
    mut rocket_cam_query: Query<&mut Camera, (With<RocketCamMarker>, Without<DroneCamMarker>)>,
    mut drone_cam_query: Query<&mut Camera, (With<DroneCamMarker>, Without<RocketCamMarker>)>,
) {
    if camera_properties.aux_cam_kind == *prev_kind {
        return;
    }
    *prev_kind = camera_properties.aux_cam_kind;

    if camera_properties.aux_cam_enabled {
        // Deactivate old, activate new
        let (activate_rq, activate_dq) = match camera_properties.aux_cam_kind {
            AuxCamKind::RocketCam => (true, false),
            AuxCamKind::DroneCam => (false, true),
        };
        if let Ok(mut cam) = rocket_cam_query.single_mut() {
            cam.is_active = activate_rq;
        }
        if let Ok(mut cam) = drone_cam_query.single_mut() {
            cam.is_active = activate_dq;
        }
    } else {
        // Both off
        if let Ok(mut cam) = rocket_cam_query.single_mut() {
            cam.is_active = false;
        }
        if let Ok(mut cam) = drone_cam_query.single_mut() {
            cam.is_active = false;
        }
    }
}

fn disable_aux_cams_on_exit(
    mut camera_properties: ResMut<CameraProperties>,
    mut rocket_cam_query: Query<&mut Camera, (With<RocketCamMarker>, Without<DroneCamMarker>)>,
    mut drone_cam_query: Query<&mut Camera, (With<DroneCamMarker>, Without<RocketCamMarker>)>,
) {
    if let Ok(mut cam) = rocket_cam_query.single_mut() {
        cam.is_active = false;
    }
    if let Ok(mut cam) = drone_cam_query.single_mut() {
        cam.is_active = false;
    }
    camera_properties.aux_cam_enabled = false;
    camera_properties.camera_swapped = false;
}

fn rocket_cam_mount_transform(rocket_dims: &RocketDimensions) -> Transform {
    // Keep the camera outside the cone mesh so fullscreen swap doesn't stare into geometry.
    let mount_pos = Vec3::new(0.0, rocket_dims.cone_length * 0.15, rocket_dims.radius + 0.08);
    Transform::from_translation(mount_pos)
        .looking_to(Vec3::new(0.0, -1.73, 1.0).normalize(), Vec3::Y)
}

fn setup_camera_system(
    mut commands: Commands,
    camera_properties: ResMut<CameraProperties>,
    asset_server: Res<AssetServer>,
    render_device: Option<Res<bevy::render::renderer::RenderDevice>>,
) {
    let camera_pos = INITIAL_CAMERA_POS;
    let camera_transform =
        Transform::from_translation(camera_pos).looking_at(camera_properties.target, Vec3::Y);

    let supported = render_device
        .map(|d| CompressedImageFormats::from_features(d.features()))
        .unwrap_or(CompressedImageFormats::NONE);

    let path = pick_best_variant(SKYBOXES[0].variants, supported);
    let skybox_handle = asset_server.load(path);

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        Camera::default(),
        Hdr,
        Tonemapping::TonyMcMapface,
        Projection::Perspective(PerspectiveProjection {
            fov: DEFAULT_FOV_DEGREES.to_radians(),
            ..default()
        }),
        Skybox {
            image: skybox_handle,
            brightness: 1000.0,
            ..default()
        },
        Bloom {
            intensity: 0.1,
            high_pass_frequency: 0.35,
            ..Bloom::NATURAL
        },
        DistanceFog {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            ..default()
        },
    ));

    // Dedicated egui overlay camera — renders UI on top of all 3D cameras.
    // Pattern from bevy_egui's split_screen example: Camera3d with alpha
    // blending output so it composites without clearing the framebuffer.
    commands.spawn((
        PrimaryEguiContext,
        camera::EguiOverlayCam,
        Camera3d::default(),
        Camera {
            order: 10,
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        RenderLayers::layer(31),
        IsDefaultUiCamera,
    ));
}


fn spawn_rocket_cam_system(
    mut commands: Commands,
    cone_query: Query<Entity, With<RocketCone>>,
    main_cam_query: Query<&Skybox, (With<Camera3d>, Without<RocketCone>, Without<camera::EguiOverlayCam>)>,
    rocket_dims: Res<RocketDimensions>,
) {
    let Ok(cone_entity) = cone_query.single() else {
        return;
    };
    let Ok(skybox) = main_cam_query.single() else {
        return;
    };
    let skybox = skybox.clone();

    let rocket_cam = commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: 1,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            Hdr,
            Tonemapping::TonyMcMapface,
            Projection::Perspective(PerspectiveProjection {
                fov: 100f32.to_radians(),
                ..default()
            }),
            rocket_cam_mount_transform(&rocket_dims),
            RocketCamMarker,
            skybox,
            Bloom {
                intensity: 0.15,
                ..Bloom::NATURAL
            },
            Name::new("RocketCam"),
        ))
        .id();

    commands.entity(cone_entity).add_child(rocket_cam);
}

fn aux_cam_viewport_resize_system(
    windows: Query<&Window>,
    camera_properties: Res<CameraProperties>,
    mut rocket_cam_query: Query<&mut Camera, (With<RocketCamMarker>, Without<DroneCamMarker>)>,
    mut drone_cam_query: Query<&mut Camera, (With<DroneCamMarker>, Without<RocketCamMarker>)>,
    mut main_cam_query: Query<&mut Camera, (With<Camera3d>, Without<RocketCamMarker>, Without<DroneCamMarker>, Without<camera::EguiOverlayCam>)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let w = window.physical_width();
    let h = window.physical_height();
    if w == 0 || h == 0 {
        return;
    }

    let pip_w = w / 4;
    let pip_h = h / 4;
    let margin = 16;
    let pip_viewport = Some(Viewport {
        physical_position: UVec2::new(w - pip_w - margin, h - pip_h - margin),
        physical_size: UVec2::new(pip_w, pip_h),
        ..default()
    });

    let active_aux_cam = match camera_properties.aux_cam_kind {
        AuxCamKind::RocketCam => rocket_cam_query.single_mut().ok(),
        AuxCamKind::DroneCam => drone_cam_query.single_mut().ok(),
    };

    if let Some(mut aux_cam) = active_aux_cam {
        if camera_properties.camera_swapped {
            aux_cam.order = 0;
            aux_cam.viewport = None;
            if let Ok(mut cam) = main_cam_query.single_mut() {
                cam.order = 1;
                cam.viewport = pip_viewport;
            }
        } else {
            if let Ok(mut cam) = main_cam_query.single_mut() {
                cam.order = 0;
                cam.viewport = None;
            }
            aux_cam.order = 1;
            aux_cam.viewport = pip_viewport;
        }
    } else if let Ok(mut cam) = main_cam_query.single_mut() {
        cam.order = 0;
        cam.viewport = None;
    }
}

fn toggle_aux_cam_system(
    mut contexts: EguiContexts,
    mut camera_properties: ResMut<CameraProperties>,
    mut rocket_cam_query: Query<&mut Camera, (With<RocketCamMarker>, Without<DroneCamMarker>)>,
    mut drone_cam_query: Query<&mut Camera, (With<DroneCamMarker>, Without<RocketCamMarker>)>,
    rocket_cam_owned: Res<save::RocketCamOwned>,
    game_mode: Res<save::GameMode>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if ctx.wants_keyboard_input() {
        return Ok(());
    }
    let v_pressed = ctx.input(|i| i.key_pressed(Key::V));
    let shift_held = ctx.input(|i| i.modifiers.shift);

    if !v_pressed {
        return Ok(());
    }

    // RocketCam requires ownership (or sandbox); DroneCam is always available
    if camera_properties.aux_cam_kind == AuxCamKind::RocketCam
        && !rocket_cam_owned.0
        && *game_mode != save::GameMode::Sandbox
    {
        return Ok(());
    }

    let set_active = |kind: AuxCamKind,
                      active: bool,
                      rq: &mut Query<&mut Camera, (With<RocketCamMarker>, Without<DroneCamMarker>)>,
                      dq: &mut Query<&mut Camera, (With<DroneCamMarker>, Without<RocketCamMarker>)>| {
        match kind {
            AuxCamKind::RocketCam => {
                if let Ok(mut cam) = rq.single_mut() {
                    cam.is_active = active;
                }
            }
            AuxCamKind::DroneCam => {
                if let Ok(mut cam) = dq.single_mut() {
                    cam.is_active = active;
                }
            }
        }
    };

    if shift_held {
        if !camera_properties.aux_cam_enabled {
            camera_properties.aux_cam_enabled = true;
            set_active(
                camera_properties.aux_cam_kind,
                true,
                &mut rocket_cam_query,
                &mut drone_cam_query,
            );
        }
        camera_properties.camera_swapped = !camera_properties.camera_swapped;
    } else {
        camera_properties.aux_cam_enabled = !camera_properties.aux_cam_enabled;
        camera_properties.camera_swapped = false;
        set_active(
            camera_properties.aux_cam_kind,
            camera_properties.aux_cam_enabled,
            &mut rocket_cam_query,
            &mut drone_cam_query,
        );
    }
    Ok(())
}

fn spawn_nav_button(parent: &mut ChildSpawnerCommands, label: &str, target: AppState) {
    parent
        .spawn((
            Button,
            NavButton(target),
            CursorIcon::System(SystemCursorIcon::Pointer),
            Node {
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(5.0), Val::Px(5.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_child((
            Text::new(label.to_string()),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        ));
}

fn handle_nav_button_clicks(
    query: Query<(&Interaction, &NavButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, nav) in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(nav.0);
        }
    }
}

fn camera_mode_button_system(
    query: Query<(&Interaction, &CameraModeButton), Changed<Interaction>>,
    mut camera_properties: ResMut<camera::CameraProperties>,
) {
    for (interaction, _) in &query {
        if *interaction == Interaction::Pressed {
            camera_properties.follow_mode = camera_properties.follow_mode.next();
        }
    }
}

fn update_camera_mode_label(
    camera_properties: Res<camera::CameraProperties>,
    mut query: Query<&mut Text, With<CameraModeLabel>>,
) {
    if !camera_properties.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("{} ", camera_properties.follow_mode.label());
    }
}

fn setup_launch_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            DespawnOnExit(AppState::Launch),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(296.0),
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(6.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_child((
            Text::new("Enter/Space: launch\nR: reset  Z: zoom\nTab: lab  Q: quit"),
            TextFont {
                font_size: 13.,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        ));

    commands
        .spawn((
            DespawnOnExit(AppState::Launch),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(80.0),
                left: Val::Px(296.0),
                column_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_nav_button(parent, "\u{2190} Lab", AppState::Lab);

            parent
                .spawn((
                    Button,
                    CameraModeButton,
                    CursorIcon::System(SystemCursorIcon::Pointer),
                    Node {
                        padding: UiRect::new(
                            Val::Px(10.0),
                            Val::Px(10.0),
                            Val::Px(5.0),
                            Val::Px(5.0),
                        ),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Free "),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                        CameraModeLabel,
                    ))
                    .with_child((
                        TextSpan::new("\u{2B6E}"),
                        TextFont {
                            font: asset_server.load("fonts/NotoSansSymbols2-Regular.ttf"),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                    ));
                });
        });

    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands
        .spawn((
            DespawnOnExit(AppState::Launch),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(12.0),
                column_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    padding: UiRect::new(
                        Val::Px(8.0),
                        Val::Px(8.0),
                        Val::Px(6.0),
                        Val::Px(8.0),
                    ),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                ))
                .with_child((
                    Text::new(""),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                    ScoreMarker,
                ));

            parent
                .spawn((Node {
                    padding: UiRect::new(
                        Val::Px(8.0),
                        Val::Px(8.0),
                        Val::Px(6.0),
                        Val::Px(8.0),
                    ),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                ))
                .with_children(|panel| {
                    panel
                        .spawn((
                            Text::new("* "),
                            TextFont {
                                font: mono_font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.9, 0.2)),
                            WindIconMarker,
                        ))
                        .with_child((
                            TextSpan::new(""),
                            TextFont {
                                font: mono_font,
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                            WindHudMarker,
                        ));
                });
        });

    commands.spawn((
        DespawnOnExit(AppState::Launch),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        CountdownOverlay,
    ))
    .with_child((
        Text::new("3"),
        TextFont {
            font_size: 120.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        CountdownDisplayMarker,
    ));
}

fn setup_lab_hud(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(AppState::Lab),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(296.0),
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(6.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_child((
            Text::new("Lab: tweak your rocket\nTab: launch pad  Q: quit"),
            TextFont {
                font_size: 13.,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        ));

    commands
        .spawn((
            DespawnOnExit(AppState::Lab),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(56.0),
                left: Val::Px(296.0),
                column_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_nav_button(parent, "\u{2190} Shop", AppState::Store);
            spawn_nav_button(parent, "Launch \u{2192}", AppState::Launch);
        });
}

#[derive(Component)]
struct StoreBalanceText;

fn setup_store_hud(mut commands: Commands, balance: Res<save::PlayerBalance>) {
    commands
        .spawn((
            DespawnOnExit(AppState::Store),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(296.0),
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(6.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_child((
            StoreBalanceText,
            Text::new(format!(
                "Store  ${:.2}\nTab: lab  Q: quit",
                balance.0
            )),
            TextFont {
                font_size: 13.,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        ));

    commands
        .spawn((
            DespawnOnExit(AppState::Store),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(56.0),
                left: Val::Px(296.0),
                column_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_nav_button(parent, "Lab \u{2192}", AppState::Lab);
        });
}

fn update_store_balance_text(
    balance: Res<save::PlayerBalance>,
    mut query: Query<&mut Text, With<StoreBalanceText>>,
) {
    if !balance.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Store  ${:.2}\nTab: lab  Q: quit", balance.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::{LAUNCH_CAMERA_POS, LAUNCH_CAMERA_TARGET};
    use bevy::ecs::message::Messages;

    fn write_message<M: Message>(app: &mut App, message: M) {
        app.world_mut().resource_mut::<Messages<M>>().write(message);
    }

    fn setup_core_app() -> App {
        let mut app = App::new();
        app.add_message::<LaunchEvent>()
            .add_message::<ResetEvent>()
            .add_message::<DownedEvent>()
            .add_message::<CollisionStart>();
        app.insert_resource(State::new(AppState::Launch));
        app.insert_resource(RocketDimensions::default());
        app.insert_resource(RocketMassModel::default());
        app.insert_resource(RocketFlightParameters::default());
        app.insert_resource(CameraProperties::default());
        app.insert_resource(RocketState::default());
        app.insert_resource(parachute::ParachuteConfig::default());
        app.insert_resource(inventory::Inventory::default());
        app.insert_resource(inventory::EquippedLoadout::default());
        app.insert_resource(save::GameMode::default());
        app.insert_resource(save::SaveState::default());
        app.insert_resource(save::PlayerBalance::default());
        app.insert_resource(save::OwnedMaterials::default());
        app.insert_resource(save::RocketCamOwned::default());
        app.insert_resource(inventory::OwnedMotorSizes::default());
        app.insert_resource(inventory::OwnedTubeTypes::default());
        app.insert_resource(inventory::OwnedNoseconeTypes::default());
        app.insert_resource(inventory::PlayerExperience::default());
        app.insert_resource(SceneCameraState::default());
        app
    }

    fn spawn_test_rocket(world: &mut World, y: f32) -> Entity {
        world
            .spawn((
                RocketMarker,
                Transform::from_xyz(0.0, y, 0.0),
                GlobalTransform::default(),
                LinearVelocity::ZERO,
                AngularVelocity::ZERO,
                LockedAxes::new(),
            ))
            .id()
    }

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "left={a}, right={b}");
    }

    #[test]
    fn launch_event_sets_state_and_force_timer() {
        let mut app = setup_core_app();
        app.add_systems(Update, on_launch_event);
        let rocket = spawn_test_rocket(app.world_mut(), 0.75);

        write_message(&mut app, LaunchEvent);
        app.update();

        let state = app.world().resource::<RocketState>();
        assert!(matches!(state.state, RocketStateEnum::Launched));
        approx_eq(state.launch_origin_y, 0.75);
        approx_eq(state.max_height, 0.0);
        approx_eq(state.max_velocity, 0.0);

        let timer = app
            .world()
            .entity(rocket)
            .get::<ForceTimer>()
            .expect("ForceTimer should be inserted on launch");
        assert!(timer.sync_rotation_with_entity);

        let locked_axes = app
            .world()
            .entity(rocket)
            .get::<LockedAxes>()
            .expect("rocket should keep LockedAxes");
        assert_eq!(locked_axes.to_bits(), LockedAxes::new().to_bits());
    }

    #[test]
    fn launch_event_ignored_when_already_launched() {
        let mut app = setup_core_app();
        app.add_systems(Update, on_launch_event);
        let rocket = spawn_test_rocket(app.world_mut(), 0.75);

        write_message(&mut app, LaunchEvent);
        app.update();
        let first_id = app
            .world()
            .entity(rocket)
            .get::<ForceTimer>()
            .expect("timer should exist after first launch")
            .id;

        write_message(&mut app, LaunchEvent);
        app.update();
        let second_id = app
            .world()
            .entity(rocket)
            .get::<ForceTimer>()
            .expect("timer should still exist after duplicate launch")
            .id;

        assert_eq!(first_id, second_id);
    }

    #[test]
    fn launch_without_rocket_is_safe() {
        let mut app = setup_core_app();
        app.add_systems(Update, on_launch_event);

        write_message(&mut app, LaunchEvent);
        app.update();

        let state = app.world().resource::<RocketState>();
        assert!(matches!(state.state, RocketStateEnum::Initial));
    }

    #[test]
    fn reset_event_restores_per_run_state() {
        let mut app = setup_core_app();
        app.add_systems(Update, on_reset_event);
        let rocket = spawn_test_rocket(app.world_mut(), 9.0);

        {
            let mut entity = app.world_mut().entity_mut(rocket);
            entity.insert(ForceTimer::default());
            entity.insert(LinearVelocity(Vec3::new(1.0, 2.0, 3.0)));
            entity.insert(AngularVelocity(Vec3::new(0.5, 0.0, -0.25)));
            entity.insert(LockedAxes::new());
            entity.insert(Transform::from_xyz(3.0, 9.0, -2.0));
        }

        {
            let mut state = app.world_mut().resource_mut::<RocketState>();
            state.state = RocketStateEnum::Launched;
            state.max_height = 42.0;
            state.max_velocity = 99.0;
            state.launch_origin_y = 10.0;
        }

        write_message(&mut app, ResetEvent);
        app.update();

        let dims = app.world().resource::<RocketDimensions>();
        let expected_y = dims.length * 0.5;
        let expected_locked_bits = lock_all_axes(LockedAxes::new()).to_bits();

        let entity = app.world().entity(rocket);
        let transform = entity
            .get::<Transform>()
            .expect("rocket should still have a transform");
        let lin = entity
            .get::<LinearVelocity>()
            .expect("rocket should still have linear velocity");
        let ang = entity
            .get::<AngularVelocity>()
            .expect("rocket should still have angular velocity");
        let locked = entity
            .get::<LockedAxes>()
            .expect("rocket should still have locked axes");

        approx_eq(transform.translation.x, 0.0);
        approx_eq(transform.translation.y, expected_y);
        approx_eq(transform.translation.z, 0.0);
        assert_eq!(*lin, LinearVelocity::ZERO);
        assert_eq!(*ang, AngularVelocity::ZERO);
        assert_eq!(locked.to_bits(), expected_locked_bits);
        assert!(!entity.contains::<ForceTimer>());

        let state = app.world().resource::<RocketState>();
        assert!(matches!(state.state, RocketStateEnum::Initial));
        approx_eq(state.max_height, 0.0);
        approx_eq(state.max_velocity, 0.0);
        approx_eq(state.launch_origin_y, expected_y);

        let camera = app.world().resource::<CameraProperties>();
        assert_eq!(camera.desired_translation, LAUNCH_CAMERA_POS);
        assert_eq!(camera.target, LAUNCH_CAMERA_TARGET);
    }

    #[test]
    fn collision_start_marks_touchdown_when_descending() {
        let mut app = setup_core_app();
        app.add_systems(Update, detect_landing_from_collision_system);
        let rocket = spawn_test_rocket(app.world_mut(), 2.0);
        let other = app.world_mut().spawn_empty().id();

        app.world_mut()
            .entity_mut(rocket)
            .insert(LinearVelocity(Vec3::new(0.0, -2.0, 0.0)));
        app.world_mut().resource_mut::<RocketState>().state = RocketStateEnum::Launched;

        write_message(
            &mut app,
            CollisionStart {
                collider1: rocket,
                collider2: other,
                body1: Some(rocket),
                body2: None,
            },
        );
        app.update();

        let state = app.world().resource::<RocketState>();
        assert!(matches!(state.state, RocketStateEnum::Grounded));
        assert_eq!(app.world().resource::<Messages<DownedEvent>>().len(), 1);
    }

    #[test]
    fn collision_start_ignored_while_ascending() {
        let mut app = setup_core_app();
        app.add_systems(Update, detect_landing_from_collision_system);
        let rocket = spawn_test_rocket(app.world_mut(), 2.0);
        let other = app.world_mut().spawn_empty().id();

        app.world_mut()
            .entity_mut(rocket)
            .insert(LinearVelocity(Vec3::new(0.0, 1.0, 0.0)));
        app.world_mut().resource_mut::<RocketState>().state = RocketStateEnum::Launched;

        write_message(
            &mut app,
            CollisionStart {
                collider1: rocket,
                collider2: other,
                body1: Some(rocket),
                body2: None,
            },
        );
        app.update();

        let state = app.world().resource::<RocketState>();
        assert!(matches!(state.state, RocketStateEnum::Launched));
        assert_eq!(app.world().resource::<Messages<DownedEvent>>().len(), 0);
    }

    #[test]
    fn rocket_position_tracks_max_metrics_relative_to_launch_origin() {
        let mut app = setup_core_app();
        app.add_systems(Update, rocket_position_system);
        let rocket = spawn_test_rocket(app.world_mut(), 12.0);
        app.world_mut()
            .entity_mut(rocket)
            .insert(LinearVelocity(Vec3::new(0.0, 4.0, 3.0)));

        {
            let mut state = app.world_mut().resource_mut::<RocketState>();
            state.state = RocketStateEnum::Launched;
            state.launch_origin_y = 2.0;
            state.max_height = 0.0;
            state.max_velocity = 0.0;
        }
        app.world_mut()
            .resource_mut::<CameraProperties>()
            .follow_mode = FollowMode::FollowSide;

        app.update();

        let state = app.world().resource::<RocketState>();
        approx_eq(state.max_height, 10.0);
        approx_eq(state.max_velocity, 5.0);
        let camera = app.world().resource::<CameraProperties>();
        approx_eq(camera.target.y, 12.0);
    }

    #[test]
    fn rocket_position_system_is_safe_without_rocket() {
        let mut app = setup_core_app();
        app.add_systems(Update, rocket_position_system);
        app.update();

        let state = app.world().resource::<RocketState>();
        assert!(matches!(state.state, RocketStateEnum::Initial));
    }
}
