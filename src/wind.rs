use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

use crate::rocket::{RocketDimensions, RocketMarker, RocketState, RocketStateEnum};

const MAX_WIND_FORCE: f32 = 0.05;
pub const MAX_WIND_SPEED_MPS: f32 = 8.0;
pub const MAX_UPDRAFT_SPEED_MPS: f32 = 2.5;

const AXIAL_FORCE_COEFF: f32 = 0.0005;
const LATERAL_FORCE_COEFF: f32 = 0.0012;
const CP_SMOOTH_RATE: f32 = 1.8;

const FREQ_1: f32 = std::f32::consts::TAU / 48.0;
const FREQ_2: f32 = std::f32::consts::TAU / 17.0;
const FREQ_3: f32 = std::f32::consts::TAU / 7.0;
const FREQ_Y_1: f32 = std::f32::consts::TAU / 23.0;
const FREQ_Y_2: f32 = std::f32::consts::TAU / 11.0;

#[derive(Resource)]
pub struct WindProperties {
    pub strength: f32,
    // UI-facing compass vector where magnitude indicates horizontal wind fraction.
    pub direction: Vec2,
    pub wind_velocity_world: Vec3,
    pub cp_offset_y_norm: f32,
    phase_x: [f32; 3],
    phase_z: [f32; 3],
    phase_y: [f32; 2],
    offset_x: [f32; 3],
    offset_z: [f32; 3],
    offset_y: [f32; 2],
    cp_target_y_norm: f32,
    cp_retarget_timer: Timer,
}

impl Default for WindProperties {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let mut rand_offset = || rng.gen_range(0.0..std::f32::consts::TAU);
        WindProperties {
            strength: 0.3,
            direction: Vec2::ZERO,
            wind_velocity_world: Vec3::ZERO,
            cp_offset_y_norm: 0.0,
            phase_x: [0.0; 3],
            phase_z: [0.0; 3],
            phase_y: [0.0; 2],
            offset_x: [rand_offset(), rand_offset(), rand_offset()],
            offset_z: [rand_offset(), rand_offset(), rand_offset()],
            offset_y: [rand_offset(), rand_offset()],
            cp_target_y_norm: rng.gen_range(-0.6..0.6),
            cp_retarget_timer: Timer::new(
                Duration::from_secs_f32(rng.gen_range(0.5..1.4)),
                TimerMode::Once,
            ),
        }
    }
}

pub fn update_wind_system(time: Res<Time>, mut wind: ResMut<WindProperties>) {
    let dt = time.delta_secs();
    let freqs = [FREQ_1, FREQ_2, FREQ_3];
    let weights = [1.0_f32, 0.6, 0.3];

    for (phase, freq) in wind.phase_x.iter_mut().zip(&freqs) {
        *phase += freq * dt;
    }
    for (phase, freq) in wind.phase_z.iter_mut().zip(&freqs) {
        *phase += freq * dt;
    }
    for (phase, freq) in wind.phase_y.iter_mut().zip([FREQ_Y_1, FREQ_Y_2]) {
        *phase += freq * dt;
    }

    let mut dx = 0.0_f32;
    let mut dz = 0.0_f32;
    for (i, weight) in weights.iter().enumerate() {
        dx += weight * (wind.phase_x[i] + wind.offset_x[i]).sin();
        dz += weight * (wind.phase_z[i] + wind.offset_z[i]).sin();
    }

    let weight_sum: f32 = weights.iter().sum();
    let horizontal_field = Vec2::new(dx / weight_sum, dz / weight_sum);
    let horiz_mag = horizontal_field.length().min(1.0);
    let horiz_dir = horizontal_field.try_normalize().unwrap_or(Vec2::ZERO);

    let vertical_field = 0.65 * (wind.phase_y[0] + wind.offset_y[0]).sin()
        + 0.35 * (wind.phase_y[1] + wind.offset_y[1]).sin();
    let horizontal_speed = wind.strength * MAX_WIND_SPEED_MPS * horiz_mag;
    let vertical_speed = wind.strength * MAX_UPDRAFT_SPEED_MPS * vertical_field;
    wind.wind_velocity_world = Vec3::new(
        horiz_dir.x * horizontal_speed,
        vertical_speed,
        horiz_dir.y * horizontal_speed,
    );
    wind.direction = Vec2::new(wind.wind_velocity_world.x, wind.wind_velocity_world.z)
        / MAX_WIND_SPEED_MPS.max(0.001);

    wind.cp_retarget_timer.tick(time.delta());
    if wind.cp_retarget_timer.just_finished() {
        let mut rng = rand::thread_rng();
        wind.cp_target_y_norm = rng.gen_range(-0.9..0.9);
        wind.cp_retarget_timer
            .set_duration(Duration::from_secs_f32(rng.gen_range(0.5..1.4)));
        wind.cp_retarget_timer.reset();
    }
    let smoothing = (dt * CP_SMOOTH_RATE).clamp(0.0, 1.0);
    wind.cp_offset_y_norm += (wind.cp_target_y_norm - wind.cp_offset_y_norm) * smoothing;
}

pub fn apply_wind_force_system(
    wind: Res<WindProperties>,
    rocket_state: Res<RocketState>,
    rocket_dims: Res<RocketDimensions>,
    mut query: Query<(&Transform, &LinearVelocity, Forces), With<RocketMarker>>,
) {
    if rocket_state.state != RocketStateEnum::Launched {
        return;
    }
    let Ok((transform, linear_velocity, mut forces)) = query.single_mut() else {
        return;
    };

    let rel_air_velocity = wind.wind_velocity_world - linear_velocity.0;
    let speed_sq = rel_air_velocity.length_squared();
    if speed_sq < 1e-6 {
        return;
    }

    let body_axis_world = transform.rotation * Vec3::Y;
    let axial = body_axis_world * rel_air_velocity.dot(body_axis_world);
    let lateral = rel_air_velocity - axial;
    let force = (axial * axial.length() * AXIAL_FORCE_COEFF
        + lateral * lateral.length() * LATERAL_FORCE_COEFF)
        .clamp_length_max(MAX_WIND_FORCE);

    let half_len = (rocket_dims.length + rocket_dims.cone_length) * 0.5;
    let local_offset = Vec3::Y * (wind.cp_offset_y_norm * half_len);
    let world_point = transform.translation + transform.rotation * local_offset;
    forces.apply_force_at_point(force, world_point);
}
