use bevy::prelude::*;

use bevy_firework::{
    core::{BlendMode, ParticleSpawnerBundle, ParticleSpawnerData, ParticleSpawnerSettings},
    emission_shape::EmissionShape,
};

use bevy_utilitarian::prelude::*;
use std::f32::consts::PI;

use crate::{
    rocket::{RocketDimensions, RocketFlightParameters, RocketMarker},
    LaunchEvent, ResetEvent,
};

pub struct RocketParticlesPlugin;

impl Plugin for RocketParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn, launch, timers, reset));
    }
}

#[derive(Component)]
struct ParticleTimers {
    paused: bool,
    delay: Timer,
    shut_down: Option<Timer>,
}
impl ParticleTimers {
    fn reset(&mut self) {
        self.paused = true;
        self.delay.reset();

        if let Some(ref mut shut_down) = self.shut_down {
            shut_down.reset();
        }
    }
    fn new(delay: f32, shut_down: Option<f32>) -> Self {
        Self {
            paused: true,
            delay: Timer::from_seconds(delay, TimerMode::Once),
            shut_down: shut_down.map(|duration| Timer::from_seconds(duration, TimerMode::Once)),
        }
    }
}

#[derive(Component)]
enum Particle {
    // TODO ignition-specific particles?
    Sparks,
    ActiveSmoke,
    ResidualSmoke,
}

impl From<Particle> for ParticleSpawnerSettings {
    fn from(particle: Particle) -> Self {
        match particle {
            Particle::Sparks => ParticleSpawnerSettings {
                starts_disabled: true,
                one_shot: false,
                rate: 250.0,
                emission_shape: EmissionShape::Circle {
                    normal: Vec3::Y,
                    radius: 0.01,
                },
                lifetime: RandF32::constant(0.5),
                inherit_parent_velocity: false,
                initial_velocity: RandVec3 {
                    magnitude: RandF32 { min: 8., max: 10. },
                    direction: -Vec3::Y,
                    spread: 10. / 180. * PI,
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
            },
            Particle::ActiveSmoke => ParticleSpawnerSettings {
                starts_disabled: true,
                one_shot: false,
                rate: 100.0,
                emission_shape: EmissionShape::Circle {
                    normal: Vec3::Y,
                    radius: 0.05,
                },
                lifetime: RandF32 { min: 8., max: 12. },
                initial_velocity: RandVec3 {
                    magnitude: RandF32 { min: 0., max: 1. },
                    direction: Vec3::Y,
                    spread: 40. / 180. * PI,
                },
                inherit_parent_velocity: false,
                initial_scale: RandF32 {
                    min: 0.04,
                    max: 0.05,
                },
                scale_curve: ParamCurve::linear(vec![(0., 1.), (0.05, 10.), (1., 50.)]),
                color: Gradient::linear(vec![
                    (0., Color::rgba(1.0, 1.0, 1.0, 0.0).into()),
                    (0.1, Color::rgba(1.0, 1.0, 1.0, 0.5).into()),
                    (1., Color::rgba(1.0, 1.0, 1.0, 0.0).into()),
                ]),
                blend_mode: BlendMode::Blend,
                linear_drag: 0.8,
                acceleration: Vec3::ZERO,
                pbr: false,
                ..default()
            },
            Particle::ResidualSmoke => ParticleSpawnerSettings {
                starts_disabled: true,
                one_shot: false,
                rate: 50.0,
                emission_shape: EmissionShape::Circle {
                    normal: Vec3::Y,
                    radius: 0.05,
                },
                lifetime: RandF32 { min: 4., max: 8. },
                initial_velocity: RandVec3 {
                    magnitude: RandF32 { min: 0., max: 1. },
                    direction: Vec3::Y,
                    spread: 40. / 180. * PI,
                },
                inherit_parent_velocity: false,
                initial_scale: RandF32 {
                    min: 0.04,
                    max: 0.05,
                },
                scale_curve: ParamCurve::linear(vec![(0., 1.), (0.05, 10.), (1., 50.)]),
                color: Gradient::linear(vec![
                    (0., Color::rgba(1.0, 1.0, 1.0, 0.0).into()),
                    (0.1, Color::rgba(1.0, 1.0, 1.0, 0.01).into()),
                    (1., Color::rgba(1.0, 1.0, 1.0, 0.0).into()),
                ]),
                blend_mode: BlendMode::Blend,
                linear_drag: 0.8,
                acceleration: Vec3::Y * 1.0,
                pbr: false,
                ..default()
            },
        }
    }
}

fn spawn(
    query: Query<Entity, Added<RocketMarker>>,
    mut commands: Commands,
    rocket_dims: Res<RocketDimensions>,
    rocket_flight_parameters: ResMut<RocketFlightParameters>,
) {
    for rocket_ent in &query {
        let sparks = commands
            .spawn(ParticleSpawnerBundle::from_settings(
                Particle::Sparks.into(),
            ))
            .insert((
                Transform::from_xyz(0., -rocket_dims.total_length() * 0.5, 0.0),
                ParticleTimers::new(0., Some(rocket_flight_parameters.duration)),
            ))
            .id();

        let active_smoke = commands
            .spawn(ParticleSpawnerBundle::from_settings(
                Particle::ActiveSmoke.into(),
            ))
            .insert((
                Transform::from_xyz(0., -rocket_dims.total_length() * 0.5, 0.0),
                ParticleTimers::new(0., Some(rocket_flight_parameters.duration)),
            ))
            .id();

        let residual_smoke = commands
            .spawn(ParticleSpawnerBundle::from_settings(
                Particle::ResidualSmoke.into(),
            ))
            .insert((
                Transform::from_xyz(0., -rocket_dims.total_length() * 0.5, 0.0),
                ParticleTimers::new(rocket_flight_parameters.duration, None),
            ))
            .id();

        commands
            .entity(rocket_ent)
            .push_children(&[sparks, active_smoke, residual_smoke]);
    }
}

fn launch(mut events: EventReader<LaunchEvent>, mut rocket_query: Query<&mut ParticleTimers>) {
    for _ in events.read() {
        for mut timers in &mut rocket_query {
            timers.reset();
            timers.paused = false;
        }
    }
}

fn timers(mut query: Query<(&mut ParticleTimers, &mut ParticleSpawnerData)>, time: Res<Time>) {
    for (mut timers, mut spawner_data) in &mut query {
        if timers.paused {
            continue;
        }

        timers.delay.tick(time.delta());
        if timers.delay.just_finished() {
            spawner_data.enabled = true
        }

        if let Some(deactivate) = &mut timers.shut_down {
            deactivate.tick(time.delta());
            if deactivate.just_finished() {
                spawner_data.enabled = false;
            }
        }
    }
}

fn reset(
    mut events: EventReader<ResetEvent>,
    rocket_query: Query<&Children, With<RocketMarker>>,
    mut spawner_query: Query<(&mut ParticleSpawnerData, &mut ParticleTimers)>,
) {
    // Consume all events, but only react to the event once per frame.
    if events.read().count() == 0 {
        return;
    };

    for children in &rocket_query {
        let mut iter = spawner_query.iter_many_mut(children);
        while let Some((mut spawner_data, mut timers)) = iter.fetch_next() {
            spawner_data.enabled = false;
            timers.paused = true;
            timers.reset();
        }
    }
}
