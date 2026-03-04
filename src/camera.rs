use avian3d::prelude::LinearVelocity;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use std::f32::consts::PI;

use crate::rocket::RocketMarker;

pub const INITIAL_CAMERA_TARGET: Vec3 = Vec3::ZERO;
pub const INITIAL_CAMERA_POS: Vec3 = Vec3::new(-6.0, 2.0, 4.0);

pub const CAMERA_DAMPING_RATIO: f32 = 1.0; // Critically damped by default.
pub const CAMERA_FAST_FOLLOW_FREQ_HZ: f32 = 6.5;
pub const CAMERA_FOLLOW_FREQ_HZ: f32 = 4.5;
pub const HUMAN_LOOK_FREQ_HZ: f32 = 2.8;
pub const CAMERA_MAX_SPEED: f32 = 85.0;
pub const SCROLL_ZOOM_SENSITIVITY: f32 = 0.01;
pub const ZOOM_LEVELS: &[f32] = &[0.8, 1.0, 2.0, 4.0, 8.0, 16.0];
pub const CAMERA_MODES: &[FollowMode] = &[
    FollowMode::FixedGround,
    FollowMode::FollowSide,
    FollowMode::FollowAbove,
    FollowMode::FreeLook,
];

#[derive(PartialEq, Copy, Clone)]
pub enum FollowMode {
    FreeLook,
    FixedGround,
    FollowAbove,
    FollowSide,
}

#[derive(Resource)]
pub struct CameraProperties {
    pub orbit_angle_degrees: f32,
    pub target: Vec3,
    pub target_y_offset: f32,
    pub lagged_target: Vec3,
    pub lagged_target_velocity: Vec3,
    pub desired_translation: Vec3,
    pub lagged_translation: Vec3,
    pub lagged_translation_velocity: Vec3,
    pub zoom: f32,
    pub zoom_index: usize,
    pub base_fov: f32,
    pub follow_mode: FollowMode,
    pub fixed_distance: f32,
    pub egui_has_pointer: bool,
}
impl Default for CameraProperties {
    fn default() -> Self {
        CameraProperties {
            orbit_angle_degrees: 20.0,
            desired_translation: INITIAL_CAMERA_POS,
            target: INITIAL_CAMERA_TARGET,
            target_y_offset: 0.0,
            lagged_target: INITIAL_CAMERA_TARGET,
            lagged_target_velocity: Vec3::ZERO,
            lagged_translation: INITIAL_CAMERA_POS,
            lagged_translation_velocity: Vec3::ZERO,
            zoom: 1.0,
            zoom_index: 1,
            base_fov: 60.0_f32.to_radians(),
            follow_mode: FollowMode::FixedGround,
            fixed_distance: 6.0,
            egui_has_pointer: false,
        }
    }
}

pub fn update_camera_zoom_perspective_system(
    mut query_camera: Query<&mut Projection>,
    camera_properties: Res<CameraProperties>,
) {
    let Ok(projection) = query_camera.single_mut() else {
        return;
    };
    // Assume perspective and skip orthographic cameras.
    let Projection::Perspective(persp) = projection.into_inner() else {
        return;
    };
    if camera_properties.is_changed() {
        // Zoom by changing base FOV; e.g. 0.5 .. 2.0.
        persp.fov = camera_properties.base_fov / camera_properties.zoom;
    }
}

pub fn update_camera_transform_system(
    time: Res<Time>,
    mut camera_properties: ResMut<CameraProperties>,
    mut camera_query: Query<(&Projection, &mut Transform)>,
    mut last_follow_mode: Local<Option<FollowMode>>,
    rocket_velocity_query: Query<&LinearVelocity, With<RocketMarker>>,
) {
    let Ok((projection, mut transform)) = camera_query.single_mut() else {
        return;
    };
    if !matches!(projection, Projection::Perspective(_)) {
        return;
    };

    let camera_properties = camera_properties.as_mut();

    // Update based on camera properties/follow mode

    let desired_target = camera_properties.target + Vec3::Y * camera_properties.target_y_offset;
    let camera_dist = camera_properties.fixed_distance;
    let follow_mode = camera_properties.follow_mode;

    // Re-seed spring state on mode switches so chase modes immediately acquire the rocket.
    if last_follow_mode.is_none_or(|prev| prev != follow_mode) {
        let rocket_velocity = rocket_velocity_query
            .single()
            .map(|v| v.0)
            .unwrap_or(Vec3::ZERO);
        camera_properties.lagged_target = desired_target;

        match follow_mode {
            FollowMode::FollowAbove => {
                camera_properties.lagged_target_velocity = rocket_velocity;
                camera_properties.lagged_translation_velocity = rocket_velocity;
                camera_properties.lagged_translation = Vec3::new(
                    desired_target.x + 0.1,
                    desired_target.y + camera_dist,
                    desired_target.z + 0.1,
                );
            }
            FollowMode::FollowSide => {
                camera_properties.lagged_target_velocity = rocket_velocity;
                camera_properties.lagged_translation_velocity = rocket_velocity;
                camera_properties.lagged_translation = Vec3::new(
                    desired_target.x + camera_dist,
                    desired_target.y + 0.5,
                    desired_target.z + 0.1,
                );
            }
            FollowMode::FixedGround => {
                camera_properties.lagged_target_velocity = Vec3::ZERO;
                camera_properties.lagged_translation_velocity = Vec3::ZERO;
                let angle_rad = camera_properties.orbit_angle_degrees.to_radians();
                camera_properties.lagged_translation = Vec3::new(
                    desired_target.x + camera_dist * angle_rad.sin(),
                    camera_properties.desired_translation.y,
                    desired_target.z + camera_dist * angle_rad.cos(),
                );
            }
            FollowMode::FreeLook => {
                camera_properties.lagged_target_velocity = Vec3::ZERO;
                camera_properties.lagged_translation_velocity = Vec3::ZERO;
                camera_properties.lagged_translation = camera_properties.desired_translation;
            }
        }
    }
    *last_follow_mode = Some(follow_mode);

    if follow_mode == FollowMode::FixedGround {
        {
            let (lagged_target, lagged_target_velocity) = (
                &mut camera_properties.lagged_target,
                &mut camera_properties.lagged_target_velocity,
            );
            spring_to_target(
                lagged_target,
                lagged_target_velocity,
                desired_target,
                HUMAN_LOOK_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }

        // Position from orbit angle and distance around target
        let angle_rad = camera_properties.orbit_angle_degrees.to_radians();
        let orbit_pos = Vec3::new(
            desired_target.x + camera_dist * angle_rad.sin(),
            camera_properties.desired_translation.y,
            desired_target.z + camera_dist * angle_rad.cos(),
        );
        {
            let (lagged_translation, lagged_translation_velocity) = (
                &mut camera_properties.lagged_translation,
                &mut camera_properties.lagged_translation_velocity,
            );
            spring_to_target(
                lagged_translation,
                lagged_translation_velocity,
                orbit_pos,
                HUMAN_LOOK_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }
    } else if follow_mode == FollowMode::FreeLook {
        {
            let (lagged_target, lagged_target_velocity) = (
                &mut camera_properties.lagged_target,
                &mut camera_properties.lagged_target_velocity,
            );
            spring_to_target(
                lagged_target,
                lagged_target_velocity,
                desired_target,
                HUMAN_LOOK_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }
        let desired = camera_properties.desired_translation;
        {
            let (lagged_translation, lagged_translation_velocity) = (
                &mut camera_properties.lagged_translation,
                &mut camera_properties.lagged_translation_velocity,
            );
            spring_to_target(
                lagged_translation,
                lagged_translation_velocity,
                desired,
                CAMERA_FOLLOW_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }
    } else if follow_mode == FollowMode::FollowAbove {
        // Interpolate look target
        {
            let (lagged_target, lagged_target_velocity) = (
                &mut camera_properties.lagged_target,
                &mut camera_properties.lagged_target_velocity,
            );
            spring_to_target(
                lagged_target,
                lagged_target_velocity,
                desired_target,
                CAMERA_FOLLOW_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }
        // Position. Actual target will be above the rocket
        {
            let (lagged_translation, lagged_translation_velocity) = (
                &mut camera_properties.lagged_translation,
                &mut camera_properties.lagged_translation_velocity,
            );
            spring_to_target(
                lagged_translation,
                lagged_translation_velocity,
                Vec3::new(
                    desired_target.x + 0.1,
                    desired_target.y + camera_dist,
                    desired_target.z + 0.1,
                ),
                CAMERA_FAST_FOLLOW_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }
    } else if follow_mode == FollowMode::FollowSide {
        // Interpolate look target
        // We want fast follow on translation but slower on look
        {
            let (lagged_target, lagged_target_velocity) = (
                &mut camera_properties.lagged_target,
                &mut camera_properties.lagged_target_velocity,
            );
            spring_to_target(
                lagged_target,
                lagged_target_velocity,
                desired_target,
                HUMAN_LOOK_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }

        // Interpolate position
        {
            let (lagged_translation, lagged_translation_velocity) = (
                &mut camera_properties.lagged_translation,
                &mut camera_properties.lagged_translation_velocity,
            );
            spring_to_target(
                lagged_translation,
                lagged_translation_velocity,
                Vec3::new(
                    desired_target.x + camera_dist,
                    desired_target.y + 0.5,
                    desired_target.z + 0.1,
                ),
                CAMERA_FAST_FOLLOW_FREQ_HZ,
                CAMERA_DAMPING_RATIO,
                CAMERA_MAX_SPEED,
                time.delta_secs(),
            );
        }
    }

    *transform = Transform::from_translation(camera_properties.lagged_translation)
        .looking_at(camera_properties.lagged_target, Vec3::Y);
}

fn spring_to_target(
    position: &mut Vec3,
    velocity: &mut Vec3,
    target: Vec3,
    frequency_hz: f32,
    damping_ratio: f32,
    max_speed: f32,
    delta_t: f32,
) {
    if delta_t <= 0.0 {
        return;
    }

    let omega = 2.0 * PI * frequency_hz.max(0.01);
    let max_step = 1.0 / 120.0;
    let steps = (delta_t / max_step).ceil().max(1.0) as usize;
    let step_dt = delta_t / steps as f32;

    for _ in 0..steps {
        let displacement = target - *position;
        let accel = displacement * (omega * omega) - *velocity * (2.0 * damping_ratio * omega);
        *velocity += accel * step_dt;

        if velocity.length_squared() > max_speed * max_speed {
            *velocity = velocity.normalize() * max_speed;
        }

        *position += *velocity * step_dt;
    }
}

pub fn mouse_orbit_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    accumulated_scroll: Res<AccumulatedMouseScroll>,
    mut camera_properties: ResMut<CameraProperties>,
) {
    if camera_properties.egui_has_pointer {
        return;
    }

    if mouse_button.pressed(MouseButton::Left) {
        let delta = accumulated_motion.delta;
        camera_properties.orbit_angle_degrees -= delta.x * 0.2;
        camera_properties.desired_translation.y -= delta.y * 0.01;
        camera_properties.desired_translation.y =
            camera_properties.desired_translation.y.clamp(0.1, 50.0);
    }

    let scroll_y = accumulated_scroll.delta.y;
    if scroll_y != 0.0 {
        camera_properties.fixed_distance -= scroll_y * SCROLL_ZOOM_SENSITIVITY;
        camera_properties.fixed_distance = camera_properties.fixed_distance.clamp(0.0, 50.0);
    }
}
