use avian3d::prelude::*;
use bevy::prelude::*;

use std::{
    hash::{Hash, Hasher},
    sync::atomic::{AtomicUsize, Ordering},
};

pub static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn get_timer_id() -> usize {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ForceTimer {
    pub id: usize,
    pub timer: Timer,
    pub force: Option<Vec3>,
    pub torque: Option<Vec3>,
    pub sync_rotation_with_entity: bool,
}
impl Hash for ForceTimer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for ForceTimer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ForceTimer {}
impl Default for ForceTimer {
    fn default() -> Self {
        ForceTimer {
            id: get_timer_id(),
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            force: None,
            torque: None,
            sync_rotation_with_entity: false,
        }
    }
}
pub fn lock_all_axes(locked_axes: LockedAxes) -> LockedAxes {
    locked_axes
        .lock_translation_x()
        .lock_translation_y()
        .lock_translation_z()
        .lock_rotation_x()
        .lock_rotation_y()
        .lock_rotation_z()
}

pub fn update_forces_system(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut query_timers: Query<(Entity, &Transform, &mut ForceTimer, Forces)>,
) {
    for (entity, ent_transform, mut force, mut forces) in query_timers.iter_mut() {
        force.timer.tick(time.delta());
        if force.timer.is_finished() {
            debug!("Timer finished, removing force timer");
            commands.entity(entity).remove::<ForceTimer>();
        } else {
            if let Some(force_vec) = force.force {
                if force.sync_rotation_with_entity {
                    forces.apply_force(ent_transform.rotation.mul_vec3(force_vec));
                } else {
                    forces.apply_force(force_vec);
                }
            }
            if let Some(torque_vec) = force.torque {
                forces.apply_torque(torque_vec);
            }
        }
    }
}
