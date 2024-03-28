use bevy::{
    app::AppExit,
    core_pipeline::{bloom::BloomSettings, Skybox},
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active,
    math::primitives::Cylinder,
    prelude::*,
    render::{
        camera::PerspectiveProjection,
        texture::{ImageAddressMode, ImageSamplerDescriptor},
    },
    transform::TransformSystem,
};
use bevy_firework::plugin::ParticleSystemPlugin;

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_generative::terrain::TerrainPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::{math::*, prelude::*};

use egui::Key;
use particles::RocketParticlesPlugin;
use sky::SkyProperties;

use crate::{
    camera::{
        set_camera_viewports,
        //update_camera_target,
        update_camera_transform_system,
        update_camera_zoom_perspective_system,
        CameraProperties,
        ControlMode,
        FollowMode,
        OccupiedScreenSpace,
        OriginalCameraTransform,
        CAMERA_MODES,
        INITIAL_CAMERA_POS,
        ZOOM_LEVELS,
    },
    cone::Cone,
    fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter},
    ground::setup_ground_system,
    physics::{get_timer_id, lock_all_axes, update_forces_system, ForceTimer, TimedForces},
    rocket::{
        create_rocket_fin_pbr_bundles, spawn_rocket_system, FinMarker, RocketBody, RocketCone,
        RocketDimensions, RocketFlightParameters, RocketMarker, RocketState, RocketStateEnum,
    },
    sky::{
        animate_light_direction, cubemap_asset_loaded, setup_sky_system, spawn_regular_sky_map,
        toggle_fog_system, CUBEMAPS, CUBEMAP_IDX,
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Initial,
}

#[derive(Event)]
struct LaunchEvent;

#[derive(Event)]
struct DownedEvent;

#[derive(Event, Default)]
struct ResetEvent;

#[derive(Component, Default)]
struct ScoreMarker;

#[derive(Bundle)]
struct EnvironmentBundle {
    camera_bundle: Camera3dBundle,
    skybox: Skybox,
    fog: FogSettings,
    bloom: BloomSettings,
}

fn main() {
    let mut app = App::new();
    app.init_state::<GameState>();
    app.add_event::<LaunchEvent>();
    app.add_event::<DownedEvent>();
    app.add_event::<ResetEvent>();

    app.add_plugins(
        DefaultPlugins
            // This is needed for tiling textures.
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    address_mode_w: ImageAddressMode::Repeat,
                    //mag_filter: ImageFilterMode::Linear,
                    //min_filter: ImageFilterMode::Linear,
                    //mipmap_filter: ImageFilterMode::Linear,
                    //lod_min_clamp: 0.,
                    //lod_max_clamp: 4.,
                    ..default()
                },
            }), /*
                .set(AssetPlugin {
                        mode: AssetMode::Processed,
                        ..default()
                    }
                ) */
    )
    //.insert_resource(ClearColor(Color::BLACK))
    .add_plugins(ParticleSystemPlugin)
    .add_plugins(PhysicsPlugins::default())
    .insert_resource(SkyProperties::default())
    .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 1.0))
    .add_plugins(EguiPlugin)
    .add_plugins(TerrainPlugin)
    .add_plugins(RocketParticlesPlugin)
    .add_plugins(FrameTimeDiagnosticsPlugin)
    .register_type::<ForceTimer>() // you need to register your type to display it
    .add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
    )
    //.add_plugins(PhysicsDebugPlugin::default())
    .init_resource::<OccupiedScreenSpace>()
    .init_resource::<RocketDimensions>()
    .init_resource::<RocketFlightParameters>()
    .init_resource::<CameraProperties>()
    .init_resource::<RocketState>()
    .add_systems(
        Startup,
        (
            //setup_effects_system,
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
        Update,
        (
            //print_collider_masses,
            ui_system,
            set_camera_viewports,
            update_rocket_dimensions_system,
            toggle_fog_system,
            update_rocket_ccd_system,
            remove_rocket_ccd_system,
            init_egui_ui_input_system,
            do_launch_system,
            //run_rocket_thrust_effect_system,
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
            //update_camera_target,
            update_camera_zoom_perspective_system,
            update_camera_transform_system,
        )
            .after(PhysicsSet::Sync)
            .before(TransformSystem::TransformPropagate),
    );

    // Simple sky box
    //app.init_resource::<FocalPoint>();
    //app.add_systems(Startup, spawn_simple_sky_box);
    //app.add_systems(Update, (sync_sky_box_center_offset));

    // Cubemap sky
    app.add_systems(Startup, spawn_regular_sky_map);
    app.add_systems(Update, (cubemap_asset_loaded, animate_light_direction));
    app.add_systems(Update, adjust_time_scale);

    #[cfg(target_arch = "wasm32")]
    app.insert_resource(Msaa::Off);

    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(Msaa::default());

    app.run();
}

fn adjust_time_scale(
    mut slowmo: Local<bool>,
    mut time: ResMut<Time<Virtual>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        *slowmo = !*slowmo;
        println!("Slowmo is {}", *slowmo);
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
            &mut TimedForces,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        (With<RocketMarker>, Without<Camera>),
    >,
    mut rocket_force_query: Query<(&mut ExternalForce, &mut ExternalTorque), With<RocketMarker>>,
    _camera_query: Query<&mut Transform, With<Camera>>,
    mut exit: EventWriter<AppExit>,
    mut camera_properties: ResMut<CameraProperties>,
    mut reset: EventWriter<ResetEvent>,
) {
    let ctx = contexts.ctx_mut();
    let (rocket_ent, mut forces, mut rocket_transform, mut lin_velocity, mut ang_velocity) =
        rocket_query.single_mut();
    if ctx.input(|i| i.key_pressed(Key::Q)) {
        exit.send(AppExit);
    }
    // Reset
    if ctx.input(|i| i.key_pressed(Key::R)) {
        println!("Resetting rocket state");

        camera_properties.desired_translation = INITIAL_CAMERA_POS;

        // Reset stats
        //rocket_state.max_height = 0;

        rocket_state.state = RocketStateEnum::Initial;

        // Position and velocity
        rocket_transform.translation = Vec3::new(0.0, rocket_dims.total_length() * 0.5, 0.0);
        rocket_transform.rotation = Quat::IDENTITY;
        *lin_velocity = LinearVelocity::ZERO;
        *ang_velocity = AngularVelocity::ZERO;

        // Clear forces. This might no longer be necessary
        let (mut ext_force, mut ext_torque) = rocket_force_query.get_single_mut().unwrap();
        forces.forces_set.clear();
        ext_force.clear();
        ext_torque.clear();

        for mut locked_axes in locked_axes.iter_mut() {
            println!("Lock axes!");
            *locked_axes = lock_all_axes(LockedAxes::new());
        }

        reset.send_default();
    }
    // TODO: Fail mode F

    // Destabilize the rocket by adding random force and torque
    if ctx.input(|input| input.key_pressed(Key::D)) {
        let destabilize_force_magnitude: f32 = 0.01;
        let destabilize_proba: f32 = 0.3;
        let destabilize_torque_proba: f32 = 0.7;
        let destabilize_torque_magnitude: f32 = 0.001;
        let destabilize_duration: f32 = 0.5 * rand::random::<f32>();

        let (_ext_force, _ext_torque) = rocket_force_query.get_single_mut().unwrap();

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
        forces.forces_set.clear();
        commands.entity(rocket_ent).remove::<ForceTimer>();
        commands.entity(rocket_ent).remove::<TimedForces>();
        rocket_transform.rotation = Quat::IDENTITY;
        *lin_velocity = LinearVelocity::ZERO;
        *ang_velocity = AngularVelocity::ZERO;
    }
}

fn do_launch_system(
    mut contexts: EguiContexts,
    _rocket_state: ResMut<RocketState>,
    mut camera_properties: ResMut<CameraProperties>,
    mut launch_event_writer: EventWriter<LaunchEvent>,
    _rocket_query: Query<(&Transform, &LinearVelocity), (With<RocketMarker>, Without<Camera>)>,
) {
    let ctx = contexts.ctx_mut();

    if ctx.input(|i| i.key_pressed(Key::C)) {
        // toggle Camera follow mode
        let idx = (camera_properties.follow_mode as usize) + 1;
        camera_properties.follow_mode = CAMERA_MODES[idx % CAMERA_MODES.len()];
    }

    if ctx.input(|i| i.key_pressed(Key::Z)) {
        camera_properties.zoom_index = (camera_properties.zoom_index + 1) % ZOOM_LEVELS.len();
        camera_properties.zoom = ZOOM_LEVELS[camera_properties.zoom_index];
    }

    if ctx.input(|i| i.key_down(Key::ArrowLeft)) {
        // Steer Rocket
        if camera_properties.control_mode == ControlMode::SteerRocket {
            // TODO: adjust rocket force vector
        } else {
            // Orbit camera
            camera_properties.orbit_angle_degrees -= 0.5;
            if camera_properties.orbit_angle_degrees < 0.0 {
                camera_properties.orbit_angle_degrees = 360.0;
            }
        }
    } else if ctx.input(|i| i.key_down(Key::ArrowRight)) {
        if camera_properties.control_mode == ControlMode::SteerRocket {
            // TODO: adjust rocket force vector
        } else {
            camera_properties.orbit_angle_degrees += 0.5;
            if camera_properties.orbit_angle_degrees > 360.0 {
                camera_properties.orbit_angle_degrees = 0.0;
            }
        }
    } else if ctx.input(|i| i.key_down(Key::ArrowUp)) {
        //camera_properties.translation.y += 0.1;
        camera_properties.fixed_distance -= 0.1;
    } else if ctx.input(|i| i.key_down(Key::ArrowDown)) {
        //camera_properties.translation.y -= 0.1;
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
        println!("Begin launch sequence!");
        launch_event_writer.send(LaunchEvent);
    }
}

fn rocket_position_system(
    mut rocket_query: Query<(&Transform, &LinearVelocity), (With<RocketMarker>, Without<Camera>)>,
    mut crash_events: EventWriter<DownedEvent>,
    mut camera_properties: ResMut<CameraProperties>,
    mut rocket_state: ResMut<RocketState>,
) {
    let (transform, velocity) = rocket_query.single_mut();
    // Update target to rocket position
    camera_properties.target = transform.translation;

    // Handle rocket crash
    if velocity.y.abs() < 1.0
        && velocity.y.abs() > 0.1
        && transform.translation.y < 0.2
        && rocket_state.state == RocketStateEnum::Launched
    {
        println!("Detected rocket down at {:?}", transform.translation);
        rocket_state.state = RocketStateEnum::Grounded;
        crash_events.send(DownedEvent);
    } else {
        // Update max stats
        rocket_state.max_height = transform.translation.y.max(rocket_state.max_height);
        rocket_state.max_velocity = velocity.length().max(rocket_state.max_velocity);
    }
}

fn on_crash_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut crash_reader: EventReader<DownedEvent>,
) {
    for _event in crash_reader.read() {
        println!("Crash event!");
        let audio_bundle = AudioBundle {
            source: asset_server.load("audio/impact_wood.ogg"), // TODO load as resource
            settings: PlaybackSettings::DESPAWN,
            ..default()
        };
        commands.spawn(audio_bundle);
    }
}

fn update_rocket_ccd_system(
    _commands: Commands,
    mut rocket_query: Query<(Entity, &Transform), With<RocketMarker>>,
) {
    // enable continuous collision detection
    // prevents falling through the ground
    // don't need the performance hit most of the time
    for (_rocket_ent, transform) in rocket_query.iter_mut() {
        if transform.translation.y < 8.0 {
            // TODO: See: Grounded in examples
            //println!("Add Ccd");
            //commands.entity(rocket_ent).insert(Ccd::enabled());
        }
    }
    //Ccd::enabled(),
}

fn remove_rocket_ccd_system(
    _commands: Commands,
    mut rocket_query: Query<(Entity, &Transform), With<RocketMarker>>,
) {
    for (_rocket_ent, transform) in rocket_query.iter_mut() {
        if transform.translation.y >= 8.0 {
            // TODO: Grounded
            //println!("Remove Ccd");
            //commands.entity(rocket_ent).remove::<Ccd>();
        }
    }
}

fn on_launch_event(
    mut launch_events: EventReader<LaunchEvent>,
    mut locked_axes: Query<&mut LockedAxes, With<RocketMarker>>,
    mut rocket_state: ResMut<RocketState>,
    mut commands: Commands,
    rocket_flight_parameters: ResMut<RocketFlightParameters>,
    mut rocket_query: Query<
        (
            Entity,
            //&mut Sleeping,
            &RigidBody,
            &Transform,
            &LinearVelocity,
        ),
        (With<RocketMarker>, Without<Camera>),
    >,
    asset_server: Res<AssetServer>,
) {
    for _ in launch_events.read() {
        println!("Launch event!");
        if rocket_state.state == RocketStateEnum::Launched {
            println!("Rocket already launched");
            return;
        }

        // Unlock physics body
        for mut locked_axes in locked_axes.iter_mut() {
            println!("Unlock axes!");
            *locked_axes = LockedAxes::new();
        }

        rocket_state.state = RocketStateEnum::Launched;

        let (rocket_ent, _, _transform, _) = rocket_query.single_mut();

        // Note: Only one ExternalForce per rigid body, but can compose
        // non-persistent forces by using .apply_force or manually adding vectors

        // We want thrust parallel to rocket fuselage
        // To do that we need to synchronize it with the rocket's rotation
        // before .apply_force
        let force_timer = ForceTimer {
            id: get_timer_id(),
            timer: Timer::from_seconds(rocket_flight_parameters.duration, TimerMode::Once),
            //force: Some(_transform.rotation.mul_vec3(Vec3::Y * rocket_flight_parameters.force)), // static
            force: Some(Vec3::Y * rocket_flight_parameters.force), // since we are syncing, we should use world space here
            torque: None,
            sync_rotation_with_entity: true,
        };
        commands.entity(rocket_ent).insert(force_timer);

        // TODO: Find how to delay audio
        let audio_bundle = AudioBundle {
            source: asset_server.load("audio/air-rushes-out-fast-long.ogg"), // TODO load as resource
            settings: PlaybackSettings::DESPAWN,
            ..default()
        };
        commands.spawn(audio_bundle);
    }
}

fn update_stats_system(
    rocket_state: Res<RocketState>,
    mut text_query: Query<&mut Text, With<ScoreMarker>>,
) {
    if let Ok(mut score_text) = text_query.get_single_mut() {
        score_text.sections[0].value = format!(
            "Max altitude: {:.1}\nMax speed: {:.1}",
            rocket_state.max_height, rocket_state.max_velocity
        );
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut rocket_dims: ResMut<RocketDimensions>,
    mut rocket_flight_parameters: ResMut<RocketFlightParameters>,
    mut camera_properties: ResMut<CameraProperties>,
    mut camera_query: Query<&Transform, With<Camera>>,
    rocket_query: Query<
        (Entity, &RigidBody, &ColliderMassProperties),
        (With<RocketMarker>, Without<Camera>),
    >,
) {
    let ctx = contexts.ctx_mut();

    if let Ok(_camera_transform) = camera_query.get_single_mut() {
        occupied_screen_space.left = egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Camera");

                ui.label("Distance");
                ui.add(egui::Slider::new(
                    &mut camera_properties.fixed_distance,
                    -50.0..=50.0,
                ));
                ui.label("Orbit");
                ui.add(egui::Slider::new(
                    &mut camera_properties.orbit_angle_degrees,
                    0.0..=360.0,
                ));

                ui.label("Elevation");
                ui.add(egui::Slider::new(
                    &mut camera_properties.desired_translation.y,
                    0.1..=20.0,
                ));
                ui.label("Zoom");
                ui.add(egui::Slider::new(&mut camera_properties.zoom, 0.2..=5.0));
                ui.label("Target Y");
                ui.add(egui::Slider::new(
                    &mut camera_properties.target.y,
                    -50.0..=50.0,
                ));

                //ui.heading("Position");
                //let pos = camera_transform.translation;
                //ui.label(format!("{:.2} {:.2} {:.2}", pos.x, pos.y, pos.z));

                ui.label("Camera Mode");

                ui.radio_value(
                    &mut camera_properties.follow_mode,
                    FollowMode::FreeLook,
                    "Free Look",
                );
                ui.radio_value(
                    &mut camera_properties.follow_mode,
                    FollowMode::FixedGround,
                    "Fixed: Ground",
                );
                ui.radio_value(
                    &mut camera_properties.follow_mode,
                    FollowMode::FollowSide,
                    "Follow: Side",
                );
                ui.radio_value(
                    &mut camera_properties.follow_mode,
                    FollowMode::FollowAbove,
                    "Follow: Above",
                );

                ui.label("Control Mode");

                ui.radio_value(
                    &mut camera_properties.control_mode,
                    ControlMode::Normal,
                    "Normal",
                );

                ui.radio_value(
                    &mut camera_properties.control_mode,
                    ControlMode::SteerRocket,
                    "Steer Rocket",
                );
                ui.heading("Rocket Properties");
                ui.label("Body");
                if ui
                    .add(egui::Slider::new(&mut rocket_dims.radius, 0.025..=8.0).text("radius"))
                    .changed()
                {
                    rocket_dims.flag_changed = true;
                };
                if ui
                    .add(egui::Slider::new(&mut rocket_dims.length, 0.025..=20.0).text("length"))
                    .changed()
                {
                    rocket_dims.flag_changed = true;
                };
                if ui
                    .add(
                        egui::Slider::new(&mut rocket_dims.cone_length, 0.01..=1.5)
                            .text("cone length"),
                    )
                    .changed()
                {
                    rocket_dims.flag_changed = true;
                };
                if ui
                    .add(
                        egui::Slider::new(&mut rocket_dims.num_fins, 1.0..=8.0)
                            .step_by(1.0)
                            .text("num fins"),
                    )
                    .changed()
                {
                    rocket_dims.flag_changed = true;
                };
                if ui
                    .add(
                        egui::Slider::new(&mut rocket_dims.fin_height, 0.01..=5.0)
                            .step_by(0.1)
                            .text("fin height"),
                    )
                    .changed()
                {
                    rocket_dims.flag_changed = true;
                };
                if ui
                    .add(
                        egui::Slider::new(&mut rocket_dims.fin_length, 0.01..=5.0)
                            .step_by(0.1)
                            .text("fin length"),
                    )
                    .changed()
                {
                    rocket_dims.flag_changed = true;
                };
                ui.label("Engine");
                ui.add(
                    egui::Slider::new(&mut rocket_flight_parameters.force, 0.05..=0.5)
                        .step_by(0.01)
                        .text("force"),
                );
                ui.add(
                    egui::Slider::new(&mut rocket_flight_parameters.duration, 0.5..=10.0)
                        .text("duration"),
                );
                if let Ok(qq) = rocket_query.get_single() {
                    let (_ent, _rigid_body, mass) = qq;
                    ui.label(format!(
                        "Mass: {:.2}, ...: {:.2} {:.2} {:.2}",
                        mass.mass.0,
                        mass.center_of_mass.x,
                        mass.center_of_mass.y,
                        mass.center_of_mass.z
                    ));
                }
                ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
            })
            .response
            .rect
            .width();
    }

    //occupied_screen_space.right = 0.0;
    //occupied_screen_space.bottom = 0.0;

    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Bottom Panel");
            //let (_, rigid_body_trans, velo) = rocket_query.single();
            ui.label("TODO");
            //let pos: Vec3 = rigid_body_trans.translation;
            //let rot: Vec3 = rigid_body_trans.rotation.mul_vec3(Vec3::Y);
            //ui.label(format!(
            //    "Position, rotation, speed: {:+.2} {:+.2} {:+.2} | {:+.2} {:+.2} {:+.2} | {:+.2} {:+.2} {:+.2}",
            //    pos.x, pos.y, pos.z, rot.x, rot.y, rot.z, velo.linvel.x, velo.linvel.y, velo.linvel.z
            //));
            //ui.label(format!("Max Height: {:.2}, Speed: {:.2}", rocket_state.max_height, rocket_state.max_velocity));
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
}

fn update_rocket_dimensions_system(
    mut commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rocket_dims: ResMut<RocketDimensions>,
    mut body_query: Query<
        (&mut Handle<Mesh>, &mut Collider, &mut Transform),
        (With<RocketBody>, Without<RocketCone>),
    >,
    mut cone_query: Query<
        (&mut Handle<Mesh>, &mut Collider, &mut Transform),
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

    println!("Updating rocket dimensions");

    // Adjust the Rigid Body position
    for mut rb_transform in rb_query.iter_mut() {
        rb_transform.translation.y = rocket_dims.total_length() * 0.5;
    }

    // Update the mesh and collider to match the new dimensions
    for (mut mesh_handle, mut collider, _) in body_query.iter_mut() {
        *mesh_handle = meshes.add(
            Cylinder::new(rocket_dims.radius, rocket_dims.length)
                .mesh()
                .resolution(rocket::CIRCLE_RESOLUTION),
        );
        *collider = Collider::cylinder(rocket_dims.length, rocket_dims.radius);
    }

    for (mut mesh_handle, mut collider, mut transform) in cone_query.iter_mut() {
        *mesh_handle = meshes.add(Mesh::from(Cone {
            radius: rocket_dims.radius,
            height: rocket_dims.cone_length,
            segments: rocket::CIRCLE_RESOLUTION,
        }));
        *collider = Collider::cone(rocket_dims.cone_length, rocket_dims.radius);
        transform.translation.y = rocket_dims.total_length() * 0.5;
    }

    // Remove fins
    // TODO: Only do this if fins changed.
    for fin in fins_query.iter_mut() {
        println!("Removing fins");
        commands.entity(fin).despawn_recursive();
    }
    // Add fins
    let rocket = rocket_query.single();
    let rocket_fin_pbr_bundles =
        create_rocket_fin_pbr_bundles(materials, rocket_dims.as_ref(), meshes.as_mut(), "#339933");
    for bundle in rocket_fin_pbr_bundles {
        // Spawn the fin entities
        commands.entity(rocket).with_children(|parent| {
            parent.spawn((bundle, FinMarker));
        });
    }
    rocket_dims.flag_changed = false;
}

fn spawn_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO: Rocket effects should be parallel to the rocket fuselage
    commands.spawn(AudioBundle {
        source: asset_server.load("audio/Welcome_to_the_Lab_v1.ogg"),
        settings: PlaybackSettings::LOOP,
        ..default()
    });
}

const DEFAULT_FOV_DEGREES: f32 = 45.0;

fn setup_camera_system(
    mut commands: Commands,
    camera_properties: ResMut<CameraProperties>,
    asset_server: Res<AssetServer>,
) {
    // Set and save original camera position
    let camera_pos = INITIAL_CAMERA_POS;
    let camera_transform =
        Transform::from_translation(camera_pos).looking_at(camera_properties.target, Vec3::Y);

    commands.insert_resource(OriginalCameraTransform(camera_transform));

    let skybox_handle = asset_server.load(CUBEMAPS[CUBEMAP_IDX].0);

    commands.spawn(EnvironmentBundle {
        camera_bundle: Camera3dBundle {
            transform: camera_transform,
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            projection: PerspectiveProjection {
                fov: DEFAULT_FOV_DEGREES.to_radians(),
                ..default()
            }
            .into(),
            ..Default::default()
        },
        skybox: Skybox {
            image: skybox_handle,
            brightness: 150.0,
        },
        bloom: BloomSettings::default(),
        fog: FogSettings::default(),
    });
}

fn setup_text_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // See: https://docs.rs/bevy/latest/src/alien_cake_addict/alien_cake_addict.rs.html#235

    // example instructions
    // TODO: game states to hide/show this?
    commands.spawn(
        TextBundle::from_section(
            "Press 'R' to reset\n\
            Press 'Enter' to launch!\n\
            Press 'C' to toggle camera mode\n\
            Press 'Z' to toggle zoom\n\
            Press 'Q' to quit\n\
            Press 'D'/'S' to destabilize/stabilize\n\
            Press 'F' to toggle fog\n\
            Press 'T' to toggle fog type\n\
            Press 'Space' to toggle slowmo\n\
            Use arrow keys to move camera (wip)\n",
            TextStyle {
                font_size: 16.,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(85.0),
            left: Val::Px(220.0),
            ..default()
        }),
    );

    // scoreboard
    commands.spawn((
        TextBundle::from_section(
            "Max altitude:",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"), // Sans-Bold
                font_size: 30.0,
                color: Color::rgb(1.0, 1.0, 1.0),
                ..default()
            },
        )
        .with_text_justify(JustifyText::Right)
        .with_background_color(Color::rgba(0.3, 0.3, 1.0, 0.7))
        .with_style(Style {
            position_type: PositionType::Relative,
            top: Val::Px(20.0),
            left: Val::Px(200.0),
            width: Val::Px(300.0),
            ..default()
        }),
        ScoreMarker,
    ));
}
