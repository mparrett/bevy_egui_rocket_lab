use avian3d::prelude::LinearVelocity;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use std::f32::consts::PI;

use crate::AppState;
use crate::rocket::RocketMarker;

#[derive(Component)]
pub struct RocketCamMarker;

#[derive(Component)]
pub struct DroneCamMarker;

#[derive(Component)]
pub struct EguiOverlayCam;

#[derive(PartialEq, Copy, Clone, Default)]
pub enum AuxCamKind {
    #[default]
    RocketCam,
    DroneCam,
}

pub const INITIAL_CAMERA_TARGET: Vec3 = Vec3::ZERO;
pub const INITIAL_CAMERA_POS: Vec3 = Vec3::new(-6.0, 2.0, 4.0);

pub const DRONE_CAM_POSITION: Vec3 = Vec3::new(0.0, 50.0, 8.0);
pub const DRONE_CAM_FOV_DEGREES: f32 = 65.0;

pub const LAUNCH_CAMERA_POS: Vec3 = INITIAL_CAMERA_POS;
pub const LAUNCH_CAMERA_DISTANCE: f32 = 6.0;

pub const LAB_CAMERA_POS: Vec3 = Vec3::new(-0.6, 1.5, 1.5);
pub const LAB_CAMERA_DISTANCE: f32 = 2.5;

pub const STORE_CAMERA_POS: Vec3 = Vec3::new(-0.3, 1.3, -0.5);
pub const STORE_CAMERA_TARGET: Vec3 = Vec3::new(0.3, 1.15, -2.2);
pub const STORE_CAMERA_DISTANCE: f32 = 1.5;

pub const CAMERA_DAMPING_RATIO: f32 = 1.0; // Critically damped by default.
pub const CAMERA_FAST_FOLLOW_FREQ_HZ: f32 = 6.5;
pub const CAMERA_FOLLOW_FREQ_HZ: f32 = 4.5;
pub const HUMAN_LOOK_FREQ_HZ: f32 = 2.8;
pub const CAMERA_MAX_SPEED: f32 = 85.0;
pub const SCROLL_ZOOM_SENSITIVITY: f32 = 0.01;
pub const FREELOOK_MOVE_SPEED: f32 = 3.0;
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
    pub aux_cam_enabled: bool,
    pub camera_swapped: bool,
    pub aux_cam_kind: AuxCamKind,
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
            follow_mode: FollowMode::FreeLook,
            fixed_distance: 6.0,
            egui_has_pointer: false,
            aux_cam_enabled: false,
            camera_swapped: false,
            aux_cam_kind: AuxCamKind::default(),
        }
    }
}

#[derive(Clone)]
pub struct CameraSnapshot {
    pub desired_translation: Vec3,
    pub target: Vec3,
    pub target_y_offset: f32,
    pub orbit_angle_degrees: f32,
    pub fixed_distance: f32,
    pub follow_mode: FollowMode,
    pub zoom: f32,
    pub zoom_index: usize,
}

impl CameraProperties {
    pub fn save_snapshot(&self) -> CameraSnapshot {
        CameraSnapshot {
            desired_translation: self.desired_translation,
            target: self.target,
            target_y_offset: self.target_y_offset,
            orbit_angle_degrees: self.orbit_angle_degrees,
            fixed_distance: self.fixed_distance,
            follow_mode: self.follow_mode,
            zoom: self.zoom,
            zoom_index: self.zoom_index,
        }
    }

    pub fn restore_snapshot(&mut self, snap: &CameraSnapshot) {
        self.desired_translation = snap.desired_translation;
        self.target = snap.target;
        self.target_y_offset = snap.target_y_offset;
        self.orbit_angle_degrees = snap.orbit_angle_degrees;
        self.fixed_distance = snap.fixed_distance;
        self.follow_mode = snap.follow_mode;
        self.zoom = snap.zoom;
        self.zoom_index = snap.zoom_index;
        self.lagged_translation = snap.desired_translation;
        self.lagged_translation_velocity = Vec3::ZERO;
        self.lagged_target = snap.target;
        self.lagged_target_velocity = Vec3::ZERO;
    }

    pub fn apply_scene_defaults(&mut self, state: &AppState, rocket_target: Vec3) {
        match state {
            AppState::Lab => {
                self.follow_mode = FollowMode::FreeLook;
                self.fixed_distance = LAB_CAMERA_DISTANCE;
                self.desired_translation = LAB_CAMERA_POS;
                self.target = rocket_target;
            }
            AppState::Store => {
                self.follow_mode = FollowMode::FreeLook;
                self.fixed_distance = STORE_CAMERA_DISTANCE;
                self.desired_translation = STORE_CAMERA_POS;
                self.target = STORE_CAMERA_TARGET;
            }
            AppState::Launch | AppState::Menu => {
                self.follow_mode = FollowMode::FreeLook;
                self.fixed_distance = LAUNCH_CAMERA_DISTANCE;
                self.desired_translation = LAUNCH_CAMERA_POS;
                self.target = rocket_target;
                self.orbit_angle_degrees = 20.0;
            }
        }
        self.lagged_translation = self.desired_translation;
        self.lagged_translation_velocity = Vec3::ZERO;
        self.lagged_target = self.target;
        self.lagged_target_velocity = Vec3::ZERO;
    }
}

#[derive(Resource, Default)]
pub struct SceneCameraState {
    pub lab: Option<CameraSnapshot>,
    pub store: Option<CameraSnapshot>,
    pub launch: Option<CameraSnapshot>,
}

impl SceneCameraState {
    pub fn get(&self, state: &AppState) -> Option<&CameraSnapshot> {
        match state {
            AppState::Lab => self.lab.as_ref(),
            AppState::Store => self.store.as_ref(),
            AppState::Launch => self.launch.as_ref(),
            AppState::Menu => None,
        }
    }

    pub fn set(&mut self, state: &AppState, snap: CameraSnapshot) {
        match state {
            AppState::Lab => self.lab = Some(snap),
            AppState::Store => self.store = Some(snap),
            AppState::Launch => self.launch = Some(snap),
            AppState::Menu => {}
        }
    }

    pub fn clear(&mut self, state: &AppState) {
        match state {
            AppState::Lab => self.lab = None,
            AppState::Store => self.store = None,
            AppState::Launch => self.launch = None,
            AppState::Menu => {}
        }
    }
}

pub fn update_camera_zoom_perspective_system(
    mut query_camera: Query<&mut Projection, (With<Camera3d>, Without<RocketCamMarker>, Without<DroneCamMarker>, Without<EguiOverlayCam>)>,
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
    mut camera_query: Query<(&Projection, &mut Transform), (With<Camera3d>, Without<RocketCamMarker>, Without<DroneCamMarker>, Without<EguiOverlayCam>)>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "left={a}, right={b}");
    }

    #[test]
    fn zoom_system_ignores_extra_cameras() {
        let mut app = App::new();
        let mut camera_properties = CameraProperties::default();
        camera_properties.zoom = 2.0;
        app.insert_resource(camera_properties);
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            RocketCamMarker,
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            DroneCamMarker,
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            EguiOverlayCam,
        ));
        app.add_systems(Update, update_camera_zoom_perspective_system);

        app.update();

        let mut projections = app.world_mut().query_filtered::<&Projection, (
            With<Camera3d>,
            Without<RocketCamMarker>,
            Without<DroneCamMarker>,
            Without<EguiOverlayCam>,
        )>();
        let projection = projections
            .single(app.world())
            .expect("expected one main 3D camera projection");
        let Projection::Perspective(persp) = projection else {
            panic!("expected perspective projection");
        };
        approx_eq(persp.fov, CameraProperties::default().base_fov / 2.0);
    }

    #[test]
    fn transform_system_ignores_extra_cameras() {
        let mut app = App::new();
        app.insert_resource(CameraProperties::default());
        app.insert_resource(Time::<()>::default());
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            Transform::IDENTITY,
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            Transform::IDENTITY,
            RocketCamMarker,
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            Transform::IDENTITY,
            DroneCamMarker,
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            Transform::IDENTITY,
            EguiOverlayCam,
        ));
        app.add_systems(Update, update_camera_transform_system);

        app.update();

        let mut query = app.world_mut().query_filtered::<&Transform, (
            With<Camera3d>,
            Without<RocketCamMarker>,
            Without<DroneCamMarker>,
            Without<EguiOverlayCam>,
        )>();
        let transform = query
            .single(app.world())
            .expect("expected one main 3D camera transform");
        assert_eq!(transform.translation, INITIAL_CAMERA_POS);
    }
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
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    accumulated_scroll: Res<AccumulatedMouseScroll>,
    mut camera_properties: ResMut<CameraProperties>,
) {
    if camera_properties.egui_has_pointer {
        return;
    }

    if mouse_button.pressed(MouseButton::Left) {
        let delta = accumulated_motion.delta;

        if camera_properties.follow_mode == FollowMode::FreeLook {
            // Mouselook: rotate the look direction around the camera position
            let cam_pos = camera_properties.desired_translation;
            let look_dir = (camera_properties.target - cam_pos).normalize_or_zero();
            if look_dir.length_squared() > 0.0 {
                let yaw = Quat::from_rotation_y(-delta.x * 0.003);
                let right = look_dir.cross(Vec3::Y).normalize_or_zero();
                let pitch = Quat::from_axis_angle(right, -delta.y * 0.003);
                let new_dir = pitch * (yaw * look_dir);
                // Clamp to avoid flipping past vertical
                if new_dir.y.abs() < 0.98 {
                    let dist = (camera_properties.target - cam_pos).length();
                    camera_properties.target = cam_pos + new_dir * dist;
                    camera_properties.lagged_target = camera_properties.target;
                }
            }
        } else {
            camera_properties.orbit_angle_degrees -= delta.x * 0.2;
            camera_properties.desired_translation.y -= delta.y * 0.01;
            camera_properties.desired_translation.y =
                camera_properties.desired_translation.y.clamp(0.1, 50.0);
        }
    }

    let scroll_y = accumulated_scroll.delta.y;
    if scroll_y != 0.0 {
        if camera_properties.follow_mode == FollowMode::FreeLook {
            // Dolly along look direction
            let cam_pos = camera_properties.desired_translation;
            let look_dir = (camera_properties.target - cam_pos).normalize_or_zero();
            let dolly = look_dir * scroll_y * 0.1;
            camera_properties.desired_translation += dolly;
            camera_properties.lagged_translation = camera_properties.desired_translation;
        } else {
            camera_properties.fixed_distance -= scroll_y * SCROLL_ZOOM_SENSITIVITY;
            camera_properties.fixed_distance = camera_properties.fixed_distance.clamp(0.0, 50.0);
        }
    }

    if camera_properties.follow_mode == FollowMode::FreeLook {
        let cam_pos = camera_properties.desired_translation;
        let look_dir = (camera_properties.target - cam_pos).normalize_or_zero();
        let forward = Vec3::new(look_dir.x, 0.0, look_dir.z).normalize_or_zero();
        let right = forward.cross(Vec3::Y).normalize_or_zero();

        let mut movement = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            movement += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement -= forward;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement += right;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            movement -= right;
        }
        if movement.length_squared() > 0.0 {
            let delta = movement.normalize() * FREELOOK_MOVE_SPEED * time.delta_secs();
            camera_properties.desired_translation += delta;
            camera_properties.target += delta;
            camera_properties.lagged_translation += delta;
            camera_properties.lagged_target += delta;
        }
    }
}
