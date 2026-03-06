use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::rocket::{RocketDimensions, RocketMarker, RocketState, RocketStateEnum};

const MAX_WIND_FORCE: f32 = 0.03;

const FREQ_1: f32 = std::f32::consts::TAU / 48.0;
const FREQ_2: f32 = std::f32::consts::TAU / 17.0;
const FREQ_3: f32 = std::f32::consts::TAU / 7.0;

#[derive(Resource)]
pub struct WindProperties {
    pub strength: f32,
    pub direction: Vec2,
    phase_x: [f32; 3],
    phase_z: [f32; 3],
    offset_x: [f32; 3],
    offset_z: [f32; 3],
}

impl Default for WindProperties {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let mut rand_offset = || rng.gen_range(0.0..std::f32::consts::TAU);
        WindProperties {
            strength: 0.3,
            direction: Vec2::ZERO,
            phase_x: [0.0; 3],
            phase_z: [0.0; 3],
            offset_x: [rand_offset(), rand_offset(), rand_offset()],
            offset_z: [rand_offset(), rand_offset(), rand_offset()],
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

    let mut dx = 0.0_f32;
    let mut dz = 0.0_f32;
    for (i, weight) in weights.iter().enumerate() {
        dx += weight * (wind.phase_x[i] + wind.offset_x[i]).sin();
        dz += weight * (wind.phase_z[i] + wind.offset_z[i]).sin();
    }

    let weight_sum: f32 = weights.iter().sum();
    wind.direction = Vec2::new(dx / weight_sum, dz / weight_sum);
}

pub fn apply_wind_force_system(
    wind: Res<WindProperties>,
    rocket_state: Res<RocketState>,
    rocket_dims: Res<RocketDimensions>,
    mut query: Query<(&Transform, Forces), With<RocketMarker>>,
) {
    if rocket_state.state != RocketStateEnum::Launched {
        return;
    }
    let Ok((transform, mut forces)) = query.single_mut() else {
        return;
    };
    let force = Vec3::new(wind.direction.x, 0.0, wind.direction.y) * wind.strength * MAX_WIND_FORCE;

    let half_len = (rocket_dims.length + rocket_dims.cone_length) * 0.5;
    let mut rng = rand::thread_rng();
    let local_offset = Vec3::Y * rng.gen_range(-half_len..half_len);
    let world_point = transform.translation + transform.rotation * local_offset;
    forces.apply_force_at_point(force, world_point);
}
