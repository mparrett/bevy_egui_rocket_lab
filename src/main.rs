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
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use egui::Key;
use particles::RocketParticlesPlugin;
use sky::SkyProperties;

use crate::{
    camera::{
        update_camera_transform_system, update_camera_zoom_perspective_system, CameraProperties,
        FollowMode, CAMERA_MODES, INITIAL_CAMERA_POS, ZOOM_LEVELS,
    },
    cone::Cone,
    fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter},
    ground::setup_ground_system,
    physics::{get_timer_id, lock_all_axes, update_forces_system, ForceTimer},
    rocket::{
        create_rocket_fin_pbr_bundles, spawn_rocket_system, FinMarker, RocketBody, RocketCone,
        RocketDimensions, RocketFlightParameters, RocketMarker, RocketState, RocketStateEnum,
    },
    sky::{
        animate_light_direction, cubemap_asset_loaded, setup_sky_system, spawn_regular_sky_map,
        toggle_fog_system, CUBEMAPS,
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

#[derive(Component, Default)]
struct ScoreMarker;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin {
        default_sampler: ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            ..default()
        },
    }));

    app.add_message::<LaunchEvent>();
    app.add_message::<DownedEvent>();
    app.add_message::<ResetEvent>();

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
                toggle_fog_system,
                update_forces_system,
                fps_text_update_system,
                fps_counter_showhide,
                update_stats_system,
                on_launch_event,
                on_crash_event,
                rocket_position_system,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                update_camera_zoom_perspective_system,
                update_camera_transform_system,
            )
                .after(PhysicsSystems::Writeback)
                .before(TransformSystems::Propagate),
        );

    app.add_systems(Startup, spawn_regular_sky_map);
    app.add_systems(Update, (cubemap_asset_loaded, animate_light_direction));
    app.add_systems(Update, adjust_time_scale);

    app.run();
}

fn adjust_time_scale(
    mut slowmo: Local<bool>,
    mut time: ResMut<Time<Virtual>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        *slowmo = !*slowmo;
        info!("Slowmo is {}", *slowmo);
    }

    if *slowmo {
        time.set_relative_speed(0.05);
    } else {
        time.set_relative_speed(1.0);
    }
}

fn init_egui_ui_input_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    rocket_dims: Res<RocketDimensions>,
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
    mut exit: MessageWriter<AppExit>,
    mut camera_properties: ResMut<CameraProperties>,
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

        camera_properties.desired_translation = INITIAL_CAMERA_POS;

        rocket_state.state = RocketStateEnum::Initial;

        // Position and velocity
        rocket_transform.translation = Vec3::new(0.0, rocket_dims.total_length() * 0.5, 0.0);
        rocket_transform.rotation = Quat::IDENTITY;
        *lin_velocity = LinearVelocity::ZERO;
        *ang_velocity = AngularVelocity::ZERO;

        // Remove any active force timers
        commands.entity(rocket_ent).remove::<ForceTimer>();

        for mut locked_axes in locked_axes.iter_mut() {
            debug!("Lock axes");
            *locked_axes = lock_all_axes(LockedAxes::new());
        }

        reset.write_default();
    }
    // TODO: Fail mode F

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
    _rocket_state: ResMut<RocketState>,
    mut camera_properties: ResMut<CameraProperties>,
    mut launch_event_writer: MessageWriter<LaunchEvent>,
    _rocket_query: Query<(&Transform, &LinearVelocity), (With<RocketMarker>, Without<Camera>)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

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

    if ctx.input(|i| i.key_down(Key::ArrowLeft)) {
        camera_properties.orbit_angle_degrees -= 0.5;
        if camera_properties.orbit_angle_degrees < 0.0 {
            camera_properties.orbit_angle_degrees = 360.0;
        }
    } else if ctx.input(|i| i.key_down(Key::ArrowRight)) {
        camera_properties.orbit_angle_degrees += 0.5;
        if camera_properties.orbit_angle_degrees > 360.0 {
            camera_properties.orbit_angle_degrees = 0.0;
        }
    } else if ctx.input(|i| i.key_down(Key::ArrowUp)) {
        camera_properties.fixed_distance -= 0.1;
    } else if ctx.input(|i| i.key_down(Key::ArrowDown)) {
        camera_properties.fixed_distance += 0.1;
    }

    // Camera truck/dolly movement
    if ctx.input(|i| i.key_down(Key::ArrowUp)) {
        let delta_to_target = camera_properties.desired_translation - camera_properties.target;
        let increment = 0.05;
        camera_properties.desired_translation.x -= increment * delta_to_target.x;
        camera_properties.desired_translation.z -= increment * delta_to_target.z;
    } else if ctx.input(|i| i.key_down(Key::ArrowDown)) {
        let delta_to_target = camera_properties.desired_translation - camera_properties.target;
        let increment = 0.05;
        camera_properties.desired_translation.x += increment * delta_to_target.x;
        camera_properties.desired_translation.z += increment * delta_to_target.z;
    }

    if ctx.input(|i| i.key_pressed(Key::Enter)) {
        info!("Begin launch sequence!");
        launch_event_writer.write(LaunchEvent);
    }
    Ok(())
}

fn rocket_position_system(
    rocket_query: Query<(&Transform, &LinearVelocity), (With<RocketMarker>, Without<Camera>)>,
    mut crash_events: MessageWriter<DownedEvent>,
    mut camera_properties: ResMut<CameraProperties>,
    mut rocket_state: ResMut<RocketState>,
) {
    let (transform, velocity) = rocket_query.single().unwrap();
    camera_properties.target = transform.translation;

    if velocity.y.abs() < 1.0
        && velocity.y.abs() > 0.1
        && transform.translation.y < 0.2
        && rocket_state.state == RocketStateEnum::Launched
    {
        info!("Detected rocket down at {:?}", transform.translation);
        rocket_state.state = RocketStateEnum::Grounded;
        crash_events.write(DownedEvent);
    } else {
        rocket_state.max_height = transform.translation.y.max(rocket_state.max_height);
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
    rocket_flight_parameters: ResMut<RocketFlightParameters>,
    mut rocket_query: Query<
        (Entity, &RigidBody, &Transform, &LinearVelocity),
        (With<RocketMarker>, Without<Camera>),
    >,
    asset_server: Res<AssetServer>,
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

        rocket_state.state = RocketStateEnum::Launched;

        let (rocket_ent, _, _transform, _) = rocket_query.single_mut().unwrap();

        let force_timer = ForceTimer {
            id: get_timer_id(),
            timer: Timer::from_seconds(rocket_flight_parameters.duration, TimerMode::Once),
            force: Some(Vec3::Y * rocket_flight_parameters.force),
            torque: None,
            sync_rotation_with_entity: true,
        };
        commands.entity(rocket_ent).insert(force_timer);

        commands.spawn((
            AudioPlayer::new(asset_server.load("audio/air-rushes-out-fast-long.ogg")),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn update_stats_system(
    rocket_state: Res<RocketState>,
    mut text_query: Query<&mut Text, With<ScoreMarker>>,
) {
    if let Ok(mut score_text) = text_query.single_mut() {
        **score_text = format!(
            "Max altitude: {:.1}\nMax speed: {:.1}",
            rocket_state.max_height, rocket_state.max_velocity
        );
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut rocket_dims: ResMut<RocketDimensions>,
    mut rocket_flight_parameters: ResMut<RocketFlightParameters>,
    mut camera_properties: ResMut<CameraProperties>,
    rocket_query: Query<(&Mass, &CenterOfMass), (With<RocketMarker>, Without<Camera>)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

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
                        .add(egui::Slider::new(&mut rocket_dims.length, 0.2..=2.0).step_by(0.05).text("length"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut rocket_dims.cone_length, 0.01..=0.8).text("cone"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut rocket_dims.num_fins, 1.0..=8.0).step_by(1.0).text("fins"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut rocket_dims.fin_height, 0.01..=1.5).step_by(0.1).text("fin H"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut rocket_dims.fin_length, 0.01..=1.0).step_by(0.1).text("fin L"))
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

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    Ok(())
}

fn update_rocket_dimensions_system(
    mut commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rocket_dims: ResMut<RocketDimensions>,
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
    let rocket = rocket_query.single().unwrap();
    let rocket_fin_pbr_bundles =
        create_rocket_fin_pbr_bundles(materials, rocket_dims.as_ref(), meshes.as_mut(), "#339933");
    for bundle in rocket_fin_pbr_bundles {
        commands.entity(rocket).with_children(|parent| {
            parent.spawn((bundle, FinMarker));
        });
    }
    rocket_dims.flag_changed = false;
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

    let (path, _) = CUBEMAPS
        .iter()
        .find(|(_, fmt)| *fmt == CompressedImageFormats::NONE || supported.contains(*fmt))
        .unwrap();

    let skybox_handle = asset_server.load(*path);

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
        DistanceFog::default(),
    ));
}

fn setup_text_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Instructions (top-right, below FPS)
    commands.spawn((
        Text::new(
            "R: reset  Enter: launch  C: camera mode\n\
             Z: zoom  Q: quit  D/S: destabilize/stabilize\n\
             F: fog  T: fog type  Space: slowmo\n\
             Arrow keys: move camera",
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
        Text::new("Max altitude:"),
        TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(6.0),
            left: Val::Px(280.0),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        ScoreMarker,
    ));
}
