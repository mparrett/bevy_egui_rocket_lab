use bevy::{
    app::AppExit,
    core_pipeline::Skybox,
    diagnostic::FrameTimeDiagnosticsPlugin,
    image::{CompressedImageFormats, ImageAddressMode, ImageSamplerDescriptor},
    input::common_conditions::input_toggle_active,
    math::primitives::Cylinder,
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_firework::plugin::ParticleSystemPlugin;

use avian3d::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use egui::Key;
use particles::RocketParticlesPlugin;
use sky::SkyProperties;

use crate::{
    camera::{
        CAMERA_MODES, CameraProperties, FollowMode, INITIAL_CAMERA_POS, ZOOM_LEVELS,
        mouse_orbit_system, update_camera_transform_system, update_camera_zoom_perspective_system,
    },
    cone::Cone,
    fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter},
    ground::setup_ground_system,
    physics::{ForceTimer, get_timer_id, lock_all_axes, update_forces_system},
    rocket::{
        FinMarker, RocketBody, RocketCone, RocketDimensions, RocketFlightParameters, RocketMarker,
        RocketState, RocketStateEnum, create_rocket_fin_pbr_bundles, spawn_rocket_system,
    },
    sky::{
        Cubemap, SKYBOXES, animate_light_direction, apply_fog_mode, cubemap_asset_loaded,
        pick_best_variant, setup_sky_system, spawn_regular_sky_map, sync_volumetrics_system,
    },
    util::random_vec,
};

mod camera;
mod cone;
mod fin;
mod fps;
mod ground;
mod particles;
mod physics;
mod rendering;
mod rocket;
mod sky;
mod util;

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

    app.add_plugins(ParticleSystemPlugin::default())
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(SkyProperties::default())
        .insert_resource(Gravity(Vec3::NEG_Y * 9.81 * 1.0))
        .add_plugins(EguiPlugin::default())
        .add_plugins(RocketParticlesPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .register_type::<ForceTimer>()
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .init_resource::<RocketDimensions>()
        .init_resource::<RocketFlightParameters>()
        .init_resource::<CameraProperties>()
        .init_resource::<RocketState>()
        .add_systems(
            Startup,
            (
                setup_ground_system,
                setup_camera_system,
                spawn_rocket_system,
                setup_sky_system,
                setup_text_system,
                setup_fps_counter,
                spawn_music,
                setup_loading_overlay,
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (ui_system, init_egui_ui_input_system, do_launch_system),
        )
        .add_systems(
            Update,
            (
                update_rocket_dimensions_system,
                update_forces_system,
                fps_text_update_system,
                fps_counter_showhide,
                on_launch_event,
                on_launch_audio_event,
                on_reset_event,
                detect_landing_from_collision_system,
                on_crash_event,
                update_stats_system,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                rocket_position_system,
                update_camera_zoom_perspective_system,
                update_camera_transform_system,
            )
                .after(PhysicsSystems::Writeback)
                .before(TransformSystems::Propagate),
        );

    app.add_systems(Startup, spawn_regular_sky_map);
    app.add_systems(
        Update,
        (
            cubemap_asset_loaded,
            animate_light_direction,
            sync_volumetrics_system,
            check_loading_complete,
        ),
    );
    app.add_systems(Update, (adjust_time_scale, mouse_orbit_system));

    app.run();
}

fn adjust_time_scale(mut time: ResMut<Time<Virtual>>, input: Res<ButtonInput<KeyCode>>) {
    // Hold backquote (`/~) for temporary slow motion.
    let slowmo_active = input.pressed(KeyCode::Backquote);
    time.set_relative_speed(if slowmo_active { 0.05 } else { 1.0 });
}

fn init_egui_ui_input_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut rocket_query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        (With<RocketMarker>, Without<Camera>),
    >,
    mut exit: MessageWriter<AppExit>,
    mut reset: MessageWriter<ResetEvent>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let (rocket_ent, mut rocket_transform, mut lin_velocity, mut ang_velocity) =
        rocket_query.single_mut()?;
    if ctx.input(|i| i.key_pressed(Key::Q)) {
        exit.write(AppExit::Success);
    }
    // Reset
    if ctx.input(|i| i.key_pressed(Key::R)) {
        info!("Resetting rocket state");
        reset.write_default();
    }
    // TODO: Add fail mode trigger

    // Destabilize the rocket by adding random force and torque
    if ctx.input(|input| input.key_pressed(Key::D)) {
        let destabilize_force_magnitude: f32 = 0.01;
        let destabilize_proba: f32 = 0.3;
        let destabilize_torque_proba: f32 = 0.7;
        let destabilize_torque_magnitude: f32 = 0.001;
        let destabilize_duration: f32 = 0.5 * rand::random::<f32>();

        let force = random_vec(destabilize_force_magnitude, destabilize_proba);
        let torque = random_vec(destabilize_torque_magnitude, destabilize_torque_proba);

        let force_timer = ForceTimer {
            id: get_timer_id(),
            timer: Timer::from_seconds(destabilize_duration, TimerMode::Once),
            force: Some(force),
            torque: Some(torque),
            sync_rotation_with_entity: false,
        };
        commands.entity(rocket_ent).insert(force_timer);
    }

    // Stabilize by resetting the forces and velocities
    if ctx.input(|i| i.key_pressed(Key::S)) {
        commands.entity(rocket_ent).remove::<ForceTimer>();
        rocket_transform.rotation = Quat::IDENTITY;
        *lin_velocity = LinearVelocity::ZERO;
        *ang_velocity = AngularVelocity::ZERO;
    }
    Ok(())
}

fn do_launch_system(
    mut contexts: EguiContexts,
    mut camera_properties: ResMut<CameraProperties>,
    mut launch_event_writer: MessageWriter<LaunchEvent>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let shift_held = ctx.input(|i| i.modifiers.shift);
    let arrow_left = ctx.input(|i| i.key_down(Key::ArrowLeft));
    let arrow_right = ctx.input(|i| i.key_down(Key::ArrowRight));
    let arrow_up = ctx.input(|i| i.key_down(Key::ArrowUp));
    let arrow_down = ctx.input(|i| i.key_down(Key::ArrowDown));

    if ctx.input(|i| i.key_pressed(Key::C)) {
        let idx = CAMERA_MODES
            .iter()
            .position(|m| *m == camera_properties.follow_mode)
            .unwrap_or(0);
        camera_properties.follow_mode = CAMERA_MODES[(idx + 1) % CAMERA_MODES.len()];
    }

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

    if arrow_up {
        if shift_held {
            // Camera truck/dolly movement in toward target.
            let delta_to_target = camera_properties.desired_translation - camera_properties.target;
            let increment = 0.05;
            camera_properties.desired_translation.x -= increment * delta_to_target.x;
            camera_properties.desired_translation.z -= increment * delta_to_target.z;
        } else {
            camera_properties.fixed_distance =
                (camera_properties.fixed_distance - 0.1).clamp(1.0, 50.0);
        }
    } else if arrow_down {
        if shift_held {
            // Camera truck/dolly movement out away from target.
            let delta_to_target = camera_properties.desired_translation - camera_properties.target;
            let increment = 0.05;
            camera_properties.desired_translation.x += increment * delta_to_target.x;
            camera_properties.desired_translation.z += increment * delta_to_target.z;
        } else {
            camera_properties.fixed_distance =
                (camera_properties.fixed_distance + 0.1).clamp(1.0, 50.0);
        }
    }

    if ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)) {
        info!("Begin launch sequence!");
        launch_event_writer.write(LaunchEvent);
    }
    Ok(())
}

fn rocket_position_system(
    rocket_query: Query<(&Transform, &LinearVelocity), (With<RocketMarker>, Without<Camera>)>,
    mut camera_properties: ResMut<CameraProperties>,
    mut rocket_state: ResMut<RocketState>,
) {
    let Ok((transform, velocity)) = rocket_query.single() else {
        return;
    };
    camera_properties.target = transform.translation;
    if rocket_state.state == RocketStateEnum::Initial {
        rocket_state.launch_origin_y = transform.translation.y;
    }

    if rocket_state.state == RocketStateEnum::Launched {
        let current_altitude = (transform.translation.y - rocket_state.launch_origin_y).max(0.0);
        rocket_state.max_height = current_altitude.max(rocket_state.max_height);
        rocket_state.max_velocity = velocity.length().max(rocket_state.max_velocity);
    }
}

fn on_crash_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut crash_reader: MessageReader<DownedEvent>,
) {
    for _event in crash_reader.read() {
        info!("Crash event");
        commands.spawn((
            AudioPlayer::new(asset_server.load("audio/impact_wood.ogg")),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn on_launch_event(
    mut launch_events: MessageReader<LaunchEvent>,
    mut locked_axes: Query<&mut LockedAxes, With<RocketMarker>>,
    mut rocket_state: ResMut<RocketState>,
    mut commands: Commands,
    rocket_flight_parameters: Res<RocketFlightParameters>,
    mut rocket_query: Query<(Entity, &Transform), (With<RocketMarker>, Without<Camera>)>,
) {
    for _ in launch_events.read() {
        info!("Launch event");
        if rocket_state.state == RocketStateEnum::Launched {
            info!("Rocket already launched");
            return;
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
        commands.entity(rocket_ent).insert(force_timer);
    }
}

fn on_launch_audio_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut launch_events: MessageReader<LaunchEvent>,
) {
    for _ in launch_events.read() {
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
) {
    if reset_events.read().next().is_none() {
        return;
    }

    *camera_properties = CameraProperties::default();
    if let Some(mut time) = virtual_time {
        time.set_relative_speed(1.0);
    }
    rocket_state.state = RocketStateEnum::Initial;
    rocket_state.max_height = 0.0;
    rocket_state.max_velocity = 0.0;

    for mut axes in &mut locked_axes {
        *axes = lock_all_axes(LockedAxes::new());
    }

    if let Ok((rocket_ent, mut rocket_transform, mut lin_velocity, mut ang_velocity)) =
        rocket_query.single_mut()
    {
        rocket_transform.translation = Vec3::new(0.0, rocket_dims.total_length() * 0.5, 0.0);
        rocket_transform.rotation = Quat::IDENTITY;
        *lin_velocity = LinearVelocity::ZERO;
        *ang_velocity = AngularVelocity::ZERO;
        rocket_state.launch_origin_y = rocket_transform.translation.y;
        camera_properties.target = rocket_transform.translation;
        camera_properties.lagged_target = rocket_transform.translation;
        camera_properties.lagged_translation = camera_properties.desired_translation;
        commands.entity(rocket_ent).remove::<ForceTimer>();
    }
}

fn detect_landing_from_collision_system(
    mut collision_events: MessageReader<CollisionStart>,
    rocket_query: Query<(Entity, &LinearVelocity), With<RocketMarker>>,
    mut rocket_state: ResMut<RocketState>,
    mut crash_events: MessageWriter<DownedEvent>,
) {
    if rocket_state.state != RocketStateEnum::Launched {
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
        "Alt: {:.1} / {:.1} m  Vel: {:.1} / {:.1} m/s",
        altitude,
        rocket_state.max_height,
        velocity.length(),
        rocket_state.max_velocity
    );
}

fn ui_system(
    mut contexts: EguiContexts,
    mut rocket_dims: ResMut<RocketDimensions>,
    mut rocket_flight_parameters: ResMut<RocketFlightParameters>,
    mut camera_properties: ResMut<CameraProperties>,
    mut sky_props: ResMut<SkyProperties>,
    rocket_query: Query<(&Mass, &CenterOfMass), (With<RocketMarker>, Without<Camera>)>,
    mut fog_query: Query<&mut DistanceFog>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    camera_properties.egui_has_pointer = ctx.wants_pointer_input();

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            egui::CollapsingHeader::new("Camera")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::Slider::new(&mut camera_properties.fixed_distance, -50.0..=50.0)
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
                    ui.add(egui::Slider::new(&mut camera_properties.zoom, 0.2..=5.0).text("zoom"));
                    ui.add(
                        egui::Slider::new(&mut camera_properties.target_y_offset, -10.0..=10.0)
                            .text("target Y"),
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
                });

            ui.add_space(6.0);
            egui::CollapsingHeader::new("Rocket Body")
                .default_open(true)
                .show(ui, |ui| {
                    let mut changed = false;
                    changed |= ui
                        .add(egui::Slider::new(&mut rocket_dims.radius, 0.025..=0.5).text("radius"))
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut rocket_dims.length, 0.2..=2.0)
                                .step_by(0.05)
                                .text("length"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut rocket_dims.cone_length, 0.01..=0.8)
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
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut rocket_dims.fin_height, 0.01..=1.5)
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
                });

            ui.add_space(6.0);
            egui::CollapsingHeader::new("Engine")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::Slider::new(&mut rocket_flight_parameters.force, 0.05..=0.5)
                            .step_by(0.01)
                            .text("force"),
                    );
                    ui.add(
                        egui::Slider::new(&mut rocket_flight_parameters.duration, 0.5..=10.0)
                            .text("duration"),
                    );
                    if let Ok((mass, com)) = rocket_query.single() {
                        ui.label(format!(
                            "Mass: {:.3}  CoM: ({:.2}, {:.2}, {:.2})",
                            mass.0, com.0.x, com.0.y, com.0.z
                        ));
                    }
                });

            ui.add_space(6.0);
            egui::CollapsingHeader::new("Sky")
                .default_open(false)
                .show(ui, |ui| {
                    let current_name = SKYBOXES[sky_props.skybox_index].name;
                    let mut changed = false;
                    egui::ComboBox::from_label("skybox")
                        .selected_text(current_name)
                        .show_ui(ui, |ui| {
                            for (i, entry) in SKYBOXES.iter().enumerate() {
                                if ui
                                    .selectable_value(&mut sky_props.skybox_index, i, entry.name)
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                        });
                    if changed {
                        sky_props.skybox_changed = true;
                        if sky_props.fog_enabled {
                            if let Ok(mut fog_settings) = fog_query.single_mut() {
                                apply_fog_mode(
                                    &mut fog_settings,
                                    sky_props.fog_mode,
                                    sky_props.fog_visibility,
                                    sky_props.skybox_index,
                                );
                            }
                        }
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

                    if sky_props.fog_enabled {
                        if ui
                            .add(
                                egui::Slider::new(&mut sky_props.fog_visibility, 10.0..=200.0)
                                    .text("visibility"),
                            )
                            .changed()
                        {
                            fog_changed = true;
                        }
                    }

                    if fog_changed {
                        if let Ok(mut fog_settings) = fog_query.single_mut() {
                            apply_fog_mode(
                                &mut fog_settings,
                                sky_props.fog_mode,
                                sky_props.fog_visibility,
                                sky_props.skybox_index,
                            );
                        }
                    }
                });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    Ok(())
}

fn update_rocket_dimensions_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rocket_dims: ResMut<RocketDimensions>,
    mut geometry_changed: MessageWriter<RocketGeometryChangedEvent>,
    mut body_query: Query<
        (&mut Mesh3d, &mut Collider, &mut Transform),
        (With<RocketBody>, Without<RocketCone>),
    >,
    mut cone_query: Query<
        (&mut Mesh3d, &mut Collider, &mut Transform),
        (With<RocketCone>, Without<RocketBody>),
    >,
    mut rb_query: Query<
        &mut Transform,
        (With<RocketMarker>, Without<RocketCone>, Without<RocketBody>),
    >,
    rocket_query: Query<Entity, With<RocketMarker>>,
    mut fins_query: Query<Entity, With<FinMarker>>,
) {
    if !rocket_dims.flag_changed {
        return;
    }

    debug!("Updating rocket dimensions");

    for mut rb_transform in rb_query.iter_mut() {
        rb_transform.translation.y = rocket_dims.total_length() * 0.5;
    }

    for (mut mesh_handle, mut collider, _) in body_query.iter_mut() {
        *mesh_handle = Mesh3d(
            meshes.add(
                Cylinder::new(rocket_dims.radius, rocket_dims.length)
                    .mesh()
                    .resolution(rocket::CIRCLE_RESOLUTION),
            ),
        );
        *collider = Collider::cylinder(rocket_dims.radius, rocket_dims.length);
    }

    for (mut mesh_handle, mut collider, mut transform) in cone_query.iter_mut() {
        *mesh_handle = Mesh3d(meshes.add(Mesh::from(Cone {
            radius: rocket_dims.radius,
            height: rocket_dims.cone_length,
            segments: rocket::CIRCLE_RESOLUTION,
        })));
        *collider = Collider::cone(rocket_dims.radius, rocket_dims.cone_length);
        transform.translation.y = rocket_dims.total_length() * 0.5;
    }

    // Remove fins
    for fin in fins_query.iter_mut() {
        debug!("Removing fins");
        commands.entity(fin).despawn();
    }
    // Add fins
    let Ok(rocket) = rocket_query.single() else {
        warn!("Rocket dimension update requested but no rocket entity exists");
        return;
    };
    let rocket_fin_pbr_bundles = create_rocket_fin_pbr_bundles(
        materials.as_mut(),
        rocket_dims.as_ref(),
        meshes.as_mut(),
        "#339933",
    );
    for bundle in rocket_fin_pbr_bundles {
        commands.entity(rocket).with_children(|parent| {
            parent.spawn((bundle, FinMarker));
        });
    }
    rocket_dims.flag_changed = false;
    geometry_changed.write_default();
}

fn spawn_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/Welcome_to_the_Lab_v1.ogg")),
        PlaybackSettings::LOOP,
    ));
}

const DEFAULT_FOV_DEGREES: f32 = 45.0;

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
        Projection::Perspective(PerspectiveProjection {
            fov: DEFAULT_FOV_DEGREES.to_radians(),
            ..default()
        }),
        Skybox {
            image: skybox_handle,
            brightness: 150.0,
            ..default()
        },
        Bloom::default(),
        DistanceFog {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            ..default()
        },
    ));
}

fn setup_text_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Instructions (top-right, below FPS)
    commands.spawn((
        Text::new(
            "R: reset  Enter/Space: launch  C: camera mode\n\
             Z: zoom  Q: quit  D/S: destabilize/stabilize\n\
             Hold `/~: slowmo  Fog: use Sky panel controls\n\
             Esc: world inspector  Arrows: orbit/dist  Shift+Up/Down: truck",
        ),
        TextFont {
            font_size: 13.,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    // Scoreboard (top-left, just past the egui panel)
    commands.spawn((
        Text::new(""),
        TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(280.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
            ..default()
        },
        ScoreMarker,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
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
        app.insert_resource(RocketDimensions::default());
        app.insert_resource(RocketFlightParameters::default());
        app.insert_resource(CameraProperties::default());
        app.insert_resource(RocketState::default());
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
        let expected_y = dims.total_length() * 0.5;
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
        assert_eq!(camera.desired_translation, INITIAL_CAMERA_POS);
        approx_eq(camera.target.y, expected_y);
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
