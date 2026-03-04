use bevy::prelude::*;

use bevy_firework::core::{
    BlendMode, EmissionPacing, EmissionSettings, ParticleSettings, ParticleSpawner,
};
use bevy_firework::curve::{FireworkCurve, FireworkGradient};
use bevy_firework::emission_shape::EmissionShape;

use bevy_utilitarian::prelude::*;
use std::f32::consts::PI;

use crate::{
    LaunchEvent, ResetEvent, RocketGeometryChangedEvent,
    rocket::{RocketDimensions, RocketFlightParameters, RocketMarker},
};

pub struct RocketParticlesPlugin;

impl Plugin for RocketParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn, launch, timers, reset, sync_emitters_with_dimensions),
        );
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
    Sparks,
    ActiveSmoke,
    ResidualSmoke,
}

impl Particle {
    fn into_spawner(self) -> ParticleSpawner {
        let (particle_settings, emission_settings) = match self {
            Particle::Sparks => (
                ParticleSettings {
                    lifetime: RandF32::constant(0.5),
                    initial_scale: RandF32 {
                        min: 0.01,
                        max: 0.05,
                    },
                    scale_curve: FireworkCurve::constant(1.),
                    base_color: FireworkGradient::uneven_samples(vec![
                        (0., LinearRgba::new(10., 7., 1., 1.)),
                        (0.7, LinearRgba::new(3., 1., 1., 1.)),
                        (0.8, LinearRgba::new(1., 0.3, 0.3, 1.)),
                        (0.9, LinearRgba::new(0.3, 0.3, 0.3, 1.)),
                        (1., LinearRgba::new(0.1, 0.1, 0.1, 0.)),
                    ]),
                    blend_mode: BlendMode::Blend,
                    linear_drag: 0.1,
                    pbr: false,
                    ..default()
                },
                EmissionSettings {
                    emission_pacing: EmissionPacing::CountOverDuration {
                        count: 200.0,
                        duration: 1.0,
                        offset_start: 0.,
                        offset_end: 1.,
                    },
                    emission_shape: EmissionShape::Circle {
                        normal: Vec3::Y,
                        radius: 0.01,
                    },
                    inherit_parent_velocity: false,
                    initial_velocity: RandVec3 {
                        magnitude: RandF32 { min: 8., max: 10. },
                        direction: -Vec3::Y,
                        spread: 10. / 180. * PI,
                    },
                    ..default()
                },
            ),
            Particle::ActiveSmoke => (
                ParticleSettings {
                    lifetime: RandF32 { min: 5., max: 8. },
                    initial_scale: RandF32 {
                        min: 0.05,
                        max: 0.065,
                    },
                    scale_curve: FireworkCurve::uneven_samples(vec![
                        (0., 1.),
                        (0.05, 8.),
                        (1., 30.),
                    ]),
                    base_color: FireworkGradient::uneven_samples(vec![
                        (0., LinearRgba::new(1.0, 1.0, 1.0, 0.0)),
                        (0.1, LinearRgba::new(1.0, 1.0, 1.0, 0.65)),
                        (1., LinearRgba::new(1.0, 1.0, 1.0, 0.0)),
                    ]),
                    blend_mode: BlendMode::Blend,
                    linear_drag: 0.8,
                    acceleration: Vec3::ZERO,
                    pbr: false,
                    ..default()
                },
                EmissionSettings {
                    emission_pacing: EmissionPacing::CountOverDuration {
                        count: 65.0,
                        duration: 1.0,
                        offset_start: 0.,
                        offset_end: 1.,
                    },
                    emission_shape: EmissionShape::Circle {
                        normal: Vec3::Y,
                        radius: 0.05,
                    },
                    initial_velocity: RandVec3 {
                        magnitude: RandF32 { min: 0., max: 1. },
                        direction: Vec3::Y,
                        spread: 40. / 180. * PI,
                    },
                    inherit_parent_velocity: false,
                    ..default()
                },
            ),
            Particle::ResidualSmoke => (
                ParticleSettings {
                    lifetime: RandF32 { min: 2.5, max: 4.0 },
                    initial_scale: RandF32 {
                        min: 0.04,
                        max: 0.05,
                    },
                    scale_curve: FireworkCurve::uneven_samples(vec![
                        (0., 1.),
                        (0.05, 8.),
                        (1., 24.),
                    ]),
                    base_color: FireworkGradient::uneven_samples(vec![
                        (0., LinearRgba::new(1.0, 1.0, 1.0, 0.0)),
                        (0.1, LinearRgba::new(1.0, 1.0, 1.0, 0.01)),
                        (1., LinearRgba::new(1.0, 1.0, 1.0, 0.0)),
                    ]),
                    blend_mode: BlendMode::Blend,
                    linear_drag: 0.8,
                    acceleration: Vec3::Y * 1.0,
                    pbr: false,
                    ..default()
                },
                EmissionSettings {
                    emission_pacing: EmissionPacing::CountOverDuration {
                        count: 25.0,
                        duration: 1.0,
                        offset_start: 0.,
                        offset_end: 1.,
                    },
                    emission_shape: EmissionShape::Circle {
                        normal: Vec3::Y,
                        radius: 0.05,
                    },
                    initial_velocity: RandVec3 {
                        magnitude: RandF32 { min: 0., max: 1. },
                        direction: Vec3::Y,
                        spread: 40. / 180. * PI,
                    },
                    inherit_parent_velocity: false,
                    ..default()
                },
            ),
        };

        ParticleSpawner {
            particle_settings: vec![particle_settings],
            emission_settings: vec![emission_settings],
            starts_enabled: false,
            ..default()
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
            .spawn((
                Particle::Sparks.into_spawner(),
                Transform::from_xyz(0., -rocket_dims.total_length() * 0.5, 0.0),
                ParticleTimers::new(0., Some(rocket_flight_parameters.duration)),
            ))
            .id();

        let active_smoke = commands
            .spawn((
                Particle::ActiveSmoke.into_spawner(),
                Transform::from_xyz(0., -rocket_dims.total_length() * 0.5, 0.0),
                ParticleTimers::new(0., Some(rocket_flight_parameters.duration)),
            ))
            .id();

        let residual_smoke = commands
            .spawn((
                Particle::ResidualSmoke.into_spawner(),
                Transform::from_xyz(0., -rocket_dims.total_length() * 0.5, 0.0),
                ParticleTimers::new(rocket_flight_parameters.duration, None),
            ))
            .id();

        commands
            .entity(rocket_ent)
            .add_children(&[sparks, active_smoke, residual_smoke]);
    }
}

fn launch(mut events: MessageReader<LaunchEvent>, mut rocket_query: Query<&mut ParticleTimers>) {
    for _ in events.read() {
        for mut timers in &mut rocket_query {
            timers.reset();
            timers.paused = false;
        }
    }
}

fn timers(mut query: Query<(&mut ParticleTimers, &mut ParticleSpawner)>, time: Res<Time>) {
    for (mut timers, mut spawner) in &mut query {
        if timers.paused {
            continue;
        }

        timers.delay.tick(time.delta());
        if timers.delay.just_finished() {
            spawner.starts_enabled = true;
        }

        if let Some(deactivate) = &mut timers.shut_down {
            deactivate.tick(time.delta());
            if deactivate.just_finished() {
                timers.paused = true;
            }
        }
    }
}

fn reset(
    mut events: MessageReader<ResetEvent>,
    rocket_query: Query<&Children, With<RocketMarker>>,
    mut spawner_query: Query<(&mut ParticleSpawner, &mut ParticleTimers)>,
) {
    if events.read().count() == 0 {
        return;
    };

    for children in &rocket_query {
        let mut iter = spawner_query.iter_many_mut(children);
        while let Some((mut spawner, mut timers)) = iter.fetch_next() {
            spawner.starts_enabled = false;
            timers.paused = true;
            timers.reset();
        }
    }
}

fn sync_emitters_with_dimensions(
    mut events: MessageReader<RocketGeometryChangedEvent>,
    rocket_query: Query<&Children, With<RocketMarker>>,
    mut particle_query: Query<&mut Transform, With<Particle>>,
    rocket_dims: Res<RocketDimensions>,
) {
    if events.read().next().is_none() {
        return;
    }

    let emitter_y = -rocket_dims.total_length() * 0.5;
    for children in &rocket_query {
        let mut iter = particle_query.iter_many_mut(children);
        while let Some(mut transform) = iter.fetch_next() {
            transform.translation = Vec3::new(0.0, emitter_y, 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::message::Messages;

    fn write_message<M: Message>(app: &mut App, message: M) {
        app.world_mut().resource_mut::<Messages<M>>().write(message);
    }

    #[test]
    fn geometry_change_reanchors_particle_emitters() {
        let mut app = App::new();
        app.add_message::<RocketGeometryChangedEvent>();
        app.insert_resource(RocketDimensions::default());
        app.add_systems(Update, sync_emitters_with_dimensions);

        let rocket = app.world_mut().spawn(RocketMarker).id();
        let child_a = app
            .world_mut()
            .spawn((Particle::Sparks, Transform::from_xyz(1.0, 2.0, 3.0)))
            .id();
        let child_b = app
            .world_mut()
            .spawn((Particle::ActiveSmoke, Transform::from_xyz(-1.0, 9.0, 0.0)))
            .id();
        let child_c = app
            .world_mut()
            .spawn((Particle::ResidualSmoke, Transform::from_xyz(0.0, -4.0, 8.0)))
            .id();
        app.world_mut()
            .entity_mut(rocket)
            .add_children(&[child_a, child_b, child_c]);

        {
            let mut dims = app.world_mut().resource_mut::<RocketDimensions>();
            dims.length = 1.6;
            dims.cone_length = 0.3;
        }

        write_message(&mut app, RocketGeometryChangedEvent);
        app.update();

        let expected_y = -0.5 * (1.6 + 0.3);
        for entity in [child_a, child_b, child_c] {
            let transform = app
                .world()
                .entity(entity)
                .get::<Transform>()
                .expect("particle child should keep transform");
            assert_eq!(transform.translation.x, 0.0);
            assert_eq!(transform.translation.z, 0.0);
            assert!(
                (transform.translation.y - expected_y).abs() < 1e-5,
                "expected y={expected_y}, got {}",
                transform.translation.y
            );
        }
    }
}
