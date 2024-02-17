use bevy::prelude::*;
use bevy_xpbd_3d::prelude::LockedAxes;
use bevy_xpbd_3d::prelude::*;

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::atomic::{AtomicUsize, Ordering},
    usize,
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
#[derive(Component, Default)]
pub struct TimedForces {
    // Using HashSet for better performance on large sets of forces
    pub forces_set: HashSet<ForceTimer>,
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
    time: Res<Time>,
    mut commands: Commands,
    mut query_timers: Query<(
        Entity,
        &mut Transform,
        &mut ForceTimer,
        &mut ExternalForce,
        &mut ExternalTorque,
    )>,
) {
    for (entity, ent_transform, mut force, mut external_force, mut external_torque) in
        query_timers.iter_mut()
    {
        force.timer.tick(time.delta());
        if force.timer.finished() {
            println!("Timer finished, removing force timer");
            commands.entity(entity).remove::<ForceTimer>();
        } else {            
            if force.force.is_some() {
                if force.sync_rotation_with_entity {
                    //println!("Applying force (synced)");
                    external_force.apply_force(
                        ent_transform.rotation.mul_vec3(Vec3::Y) * force.force.unwrap(),
                    );
                } else {
                    //println!("Applying force (synced)");
                    external_force.apply_force(force.force.unwrap());
                }
            }
            if force.torque.is_some() {
                external_torque.apply_torque(force.torque.unwrap());
            }
        }
    }
}
