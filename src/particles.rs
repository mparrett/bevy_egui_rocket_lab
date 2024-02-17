use bevy::prelude::*;

use bevy_firework::{
    core::{BlendMode, ParticleSpawnerBundle, ParticleSpawnerSettings},
    emission_shape::EmissionShape,
};

use bevy_utilitarian::prelude::*;
use std::f32::consts::PI;

#[derive(Component, Default)]
struct RocketParticle;

pub fn get_rocket_particle_spawn_bundle() -> ParticleSpawnerBundle {
    ParticleSpawnerBundle::from_settings(ParticleSpawnerSettings {
        one_shot: false,
        rate: 100.0,
        emission_shape: EmissionShape::Circle {
            normal: Vec3::Y,
            radius: 0.05,
        },
        lifetime: RandF32::constant(0.5),
        inherit_parent_velocity: true,
        initial_velocity: RandVec3 {
            magnitude: RandF32 { min: 0., max: 8. },
            direction: -Vec3::Y,
            spread: 20. / 180. * PI,
        },
        initial_scale: RandF32 {
            min: 0.01,
            max: 0.05,
        },
        scale_curve: ParamCurve::constant(1.),
        color: Gradient::linear(vec![
            (0., Color::rgba(10., 7., 1., 1.).into()),
            (0.7, Color::rgba(3., 1., 1., 1.).into()),
            (0.8, Color::rgba(1., 0.3, 0.3, 1.).into()),
            (0.9, Color::rgba(0.3, 0.3, 0.3, 1.).into()),
            (1., Color::rgba(0.1, 0.1, 0.1, 0.).into()),
        ]),
        blend_mode: BlendMode::Blend,
        linear_drag: 0.1,
        pbr: false,
        ..default()
    })
}
