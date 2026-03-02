use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

pub const INITIAL_CAMERA_TARGET: Vec3 = Vec3::ZERO;
pub const INITIAL_CAMERA_POS: Vec3 = Vec3::new(-6.0, 2.0, 4.0);

pub const CAMERA_FAST_FOLLOW_SPEED: f32 = 40.0;
pub const CAMERA_FOLLOW_SPEED: f32 = 10.0; // Faster follow speed for above/side camera
pub const HUMAN_LOOK_SPEED: f32 = 3.0; // Mimic human head movement
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
    pub desired_translation: Vec3,
    pub lagged_translation: Vec3,
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
            lagged_translation: INITIAL_CAMERA_POS,
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
    camera_properties: ResMut<CameraProperties>,
) {
    // assume perspective. do nothing if orthographic.
    let Projection::Perspective(persp) = query_camera.single_mut().unwrap().into_inner() else {
        return;
    };
    if camera_properties.is_changed() {
        // zoom in, zoom out by changing base FOV; e.g. 0.5 .. 2.0
        persp.fov = camera_properties.base_fov / camera_properties.zoom;
    }
}

pub fn update_camera_transform_system(
    time: Res<Time>,
    mut camera_properties: ResMut<CameraProperties>,
    mut camera_query: Query<(&Projection, &mut Transform)>,
) {
    let (_, mut transform) = match camera_query.single_mut() {
        Ok((Projection::Perspective(projection), transform)) => (projection, transform),
        _ => unreachable!(),
    };

    // Update based on camera properties/follow mode

    let desired_target = camera_properties.target + Vec3::Y * camera_properties.target_y_offset;
    let camera_dist = camera_properties.fixed_distance;

    if camera_properties.follow_mode == FollowMode::FixedGround {
        let spring_mu = HUMAN_LOOK_SPEED;
        interpolate_to_target(
            &mut camera_properties.lagged_target,
            desired_target,
            spring_mu,
            time.delta_secs(),
        );

        // Position from orbit angle and distance around target
        let angle_rad = camera_properties.orbit_angle_degrees.to_radians();
        let orbit_pos = Vec3::new(
            desired_target.x + camera_dist * angle_rad.sin(),
            camera_properties.desired_translation.y,
            desired_target.z + camera_dist * angle_rad.cos(),
        );
        interpolate_to_target(
            &mut camera_properties.lagged_translation,
            orbit_pos,
            spring_mu,
            time.delta_secs(),
        );
    } else if camera_properties.follow_mode == FollowMode::FreeLook {
        interpolate_to_target(
            &mut camera_properties.lagged_target,
            desired_target,
            HUMAN_LOOK_SPEED,
            time.delta_secs(),
        );
        let desired = camera_properties.desired_translation;
        interpolate_to_target(
            &mut camera_properties.lagged_translation,
            desired,
            CAMERA_FOLLOW_SPEED,
            time.delta_secs(),
        );
    } else if camera_properties.follow_mode == FollowMode::FollowAbove {
        // Interpolate look target
        interpolate_to_target(
            &mut camera_properties.lagged_target,
            desired_target,
            CAMERA_FOLLOW_SPEED,
            time.delta_secs(),
        );
        // Position. Actual target will be above the rocket
        interpolate_to_target(
            &mut camera_properties.lagged_translation,
            Vec3::new(
                desired_target.x + 0.1,
                desired_target.y + camera_dist,
                desired_target.z + 0.1,
            ),
            CAMERA_FAST_FOLLOW_SPEED,
            time.delta_secs(),
        )
    } else if camera_properties.follow_mode == FollowMode::FollowSide {
        // Interpolate look target
        // We want fast follow on translation but slower on look
        interpolate_to_target(
            &mut camera_properties.lagged_target,
            desired_target,
            HUMAN_LOOK_SPEED,
            time.delta_secs(),
        );

        // Interpolate position
        interpolate_to_target(
            &mut camera_properties.lagged_translation,
            Vec3::new(
                desired_target.x + camera_dist,
                desired_target.y + 0.5,
                desired_target.z + 0.1,
            ),
            CAMERA_FOLLOW_SPEED,
            time.delta_secs(),
        );
    }

    *transform = Transform::from_translation(camera_properties.lagged_translation)
        .looking_at(camera_properties.lagged_target, Vec3::Y);
}

fn interpolate_to_target(target: &mut Vec3, target_vec: Vec3, spring_mu: f32, delta_t: f32) {
    target.x = target.x - (target.x - target_vec.x) * spring_mu * delta_t;
    target.y = target.y - (target.y - target_vec.y) * spring_mu * delta_t;
    target.z = target.z - (target.z - target_vec.z) * spring_mu * delta_t;
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
        camera_properties.fixed_distance -= scroll_y * 0.5;
        camera_properties.fixed_distance = camera_properties.fixed_distance.clamp(1.0, 50.0);
    }
}
