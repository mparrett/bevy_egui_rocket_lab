use avian3d::prelude::*;
use bevy::prelude::*;

use crate::parachute::{DetachedCone, ParachuteConfig};
use crate::rocket::{RocketDimensions, RocketMarker, RocketState, RocketStateEnum};
use crate::wind;

const CD_AXIAL: f32 = 0.4;
const CD_LATERAL: f32 = 1.2;
const AIR_DENSITY: f32 = 1.225;
const MAX_DRAG_FORCE: f32 = 20.0;

pub fn apply_aerodynamic_drag_system(
    rocket_state: Res<RocketState>,
    rocket_dims: Res<RocketDimensions>,
    wind: Res<wind::WindProperties>,
    mut query: Query<(&Transform, Forces), With<RocketMarker>>,
) {
    if !matches!(
        rocket_state.state,
        RocketStateEnum::Launched | RocketStateEnum::Descending
    ) {
        return;
    }
    let Ok((transform, mut forces)) = query.single_mut() else {
        return;
    };

    let relative_velocity = forces.linear_velocity() - wind.wind_velocity_world;
    let speed_sq = relative_velocity.length_squared();
    if speed_sq < 1e-6 {
        return;
    }

    let body_axis = transform.rotation * Vec3::Y;

    let axial_speed = relative_velocity.dot(body_axis);
    let axial_component = body_axis * axial_speed;
    let lateral_component = relative_velocity - axial_component;

    let radius = rocket_dims.radius;
    let total_length = rocket_dims.length + rocket_dims.cone_length;

    let axial_area = std::f32::consts::PI * radius * radius;
    let axial_speed_sq = axial_component.length_squared();
    let axial_drag_mag = 0.5 * CD_AXIAL * axial_area * AIR_DENSITY * axial_speed_sq;
    let axial_drag = if axial_speed_sq > 1e-8 {
        -axial_component.normalize() * axial_drag_mag
    } else {
        Vec3::ZERO
    };

    let lateral_area = total_length * 2.0 * radius;
    let lateral_speed_sq = lateral_component.length_squared();
    let lateral_drag_mag = 0.5 * CD_LATERAL * lateral_area * AIR_DENSITY * lateral_speed_sq;
    let lateral_drag = if lateral_speed_sq > 1e-8 {
        -lateral_component.normalize() * lateral_drag_mag
    } else {
        Vec3::ZERO
    };

    let total_drag = (axial_drag + lateral_drag).clamp_length_max(MAX_DRAG_FORCE);
    forces.apply_force(total_drag);
}

const CD_CONE: f32 = 1.0;

pub fn apply_cone_drag_system(
    parachute_config: Res<ParachuteConfig>,
    rocket_dims: Res<RocketDimensions>,
    wind: Res<wind::WindProperties>,
    mut query: Query<Forces, With<DetachedCone>>,
) {
    if !parachute_config.deployed {
        return;
    }
    let Ok(mut forces) = query.single_mut() else {
        return;
    };

    let relative_velocity = forces.linear_velocity() - wind.wind_velocity_world;
    let speed_sq = relative_velocity.length_squared();
    if speed_sq < 1e-6 {
        return;
    }

    let r = rocket_dims.radius;
    let area = std::f32::consts::PI * r * r;
    let speed = speed_sq.sqrt();
    let drag_mag = (0.5 * CD_CONE * area * AIR_DENSITY * speed_sq).min(MAX_DRAG_FORCE);
    forces.apply_force(-relative_velocity / speed * drag_mag);
}
