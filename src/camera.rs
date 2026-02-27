use bevy::{
    prelude::*,
    render::camera::Projection,
    window::{PrimaryWindow, WindowResized},
};

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Resource, Deref, DerefMut)]
pub struct OriginalCameraTransform(pub Transform);

pub const INITIAL_CAMERA_TARGET: Vec3 = Vec3::ZERO;
pub const INITIAL_CAMERA_POS: Vec3 = Vec3::new(-6.0, 2.0, 4.0);

pub const CAMERA_FAST_FOLLOW_SPEED: f32 = 40.0;
pub const CAMERA_FOLLOW_SPEED: f32 = 10.0; // Faster follow speed for above/side camera
pub const HUMAN_LOOK_SPEED: f32 = 3.0; // Mimic human head movement
pub const POSITION_LAG_RATIO: f32 = 0.99;

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

//const CONTROL_MODES: &[ControlMode] = &[ControlMode::Normal, ControlMode::SteerRocket];

#[derive(PartialEq, Copy, Clone)]
pub enum ControlMode {
    Normal,
    SteerRocket,
}

#[derive(Resource)]
pub struct CameraProperties {
    pub distance_from_target: f32,
    pub orbit_angle_degrees: f32,
    pub elevation_meters: f32, // TODO: Configure units
    pub target: Vec3,
    pub lagged_target: Vec3,
    pub desired_translation: Vec3,
    pub lagged_translation: Vec3,
    pub zoom: f32,
    pub zoom_index: usize,
    pub base_fov: f32,
    pub follow_mode: FollowMode,
    pub control_mode: ControlMode,
    pub fixed_distance: f32,
}
impl Default for CameraProperties {
    fn default() -> Self {
        CameraProperties {
            distance_from_target: -25.0,
            orbit_angle_degrees: 20.0,
            elevation_meters: INITIAL_CAMERA_POS.y,
            desired_translation: INITIAL_CAMERA_POS,
            target: INITIAL_CAMERA_TARGET,
            lagged_target: INITIAL_CAMERA_TARGET,
            lagged_translation: INITIAL_CAMERA_POS,
            zoom: 1.0,
            zoom_index: 1,
            base_fov: 60.0_f32.to_radians(),
            follow_mode: FollowMode::FixedGround,
            control_mode: ControlMode::Normal,
            fixed_distance: 6.0,
        }
    }
}

pub fn update_camera_zoom_perspective_system(
    mut query_camera: Query<&mut Projection>,
    camera_properties: ResMut<CameraProperties>,
) {
    // assume perspective. do nothing if orthographic.
    let Projection::Perspective(persp) = query_camera.single_mut().into_inner() else {
        return;
    };
    if camera_properties.is_changed() {
        // zoom in, zoom out by changing base FOV; e.g. 0.5 .. 2.0
        persp.fov = camera_properties.base_fov / camera_properties.zoom;
    }
}
use bevy::render::camera::Viewport;

pub fn update_camera_transform_system(
    time: Res<Time>,
    used_screen_space: Res<OccupiedScreenSpace>,
    original_camera_transform: Res<OriginalCameraTransform>,
    mut camera_properties: ResMut<CameraProperties>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Camera, &Projection, &mut Transform)>,
) {
    let (mut camera, _, mut transform) = match camera_query.get_single_mut() {
        Ok((camera, Projection::Perspective(projection), transform)) => {
            (camera, projection, transform)
        }
        _ => unreachable!(),
    };

    // Adjust viewport based on window size and occupied screen space
    let window = windows.single();
    let size = UVec2::new(
        window.physical_width() - used_screen_space.left as u32 - used_screen_space.right as u32,
        window.physical_height() - used_screen_space.top as u32 - used_screen_space.bottom as u32,
    );

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(used_screen_space.left as u32, used_screen_space.top as u32),
        physical_size: size,
        ..default()
    });

    // Update based on camera properties/follow mode

    // Using mutable camera target
    let desired_target = camera_properties.target; // desired target
                                                   //println!("Desired target: {:?}", desired_target);
    let camera_dist = camera_properties.fixed_distance;

    if camera_properties.follow_mode == FollowMode::FixedGround {
        // Update look-at target
        // Spring/gimbal following
        let spring_mu = HUMAN_LOOK_SPEED;
        interpolate_to_target(
            &mut camera_properties.lagged_target,
            desired_target,
            spring_mu,
            time.delta_seconds(),
        );

        // Position: Original camera transform
        camera_properties.lagged_translation = original_camera_transform.translation;
        //interpolate_to_target(&mut camera_properties.lagged_translation,
        //    original_camera_transform.translation, spring_mu, time.delta_seconds());
    } else if camera_properties.follow_mode == FollowMode::FollowAbove {
        // Interpolate look target
        interpolate_to_target(
            &mut camera_properties.lagged_target,
            desired_target,
            CAMERA_FOLLOW_SPEED,
            time.delta_seconds(),
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
            time.delta_seconds(),
        )
    } else if camera_properties.follow_mode == FollowMode::FollowSide {
        // Interpolate look target
        // We want fast follow on translation but slower on look
        interpolate_to_target(
            &mut camera_properties.lagged_target,
            desired_target,
            HUMAN_LOOK_SPEED,
            time.delta_seconds(),
        );

        // Interpolate position
        interpolate_to_target_alt(
            &mut camera_properties.lagged_translation,
            Vec3::new(
                desired_target.x + camera_dist,
                desired_target.y + 0.5,
                desired_target.z + 0.1,
            ),
            POSITION_LAG_RATIO,
            time.delta_seconds(),
        );

        // Handle orbit but with correct math
        // let angle = camera_properties.orbit_angle_degrees.to_radians();
        // camera_properties.translation = camera_properties.target
        //     + Vec3::new(
        //         angle.sin() * (camera_properties.fixed_distance + camera_properties.target.x),
        //         camera_properties.translation.y,
        //         angle.cos() * (camera_properties.fixed_distance + camera_properties.target.z),
        //     );
    }

    //println!("Lagged target: {:?}", camera_properties.lagged_target);
    // Update camera transform based on dynamic target and position
    // println!("{}", translation);
    *transform = Transform::from_translation(camera_properties.lagged_translation)
        //Transform::from_translation(original_camera_transform.translation)
        .looking_at(camera_properties.lagged_target, Vec3::Y);
}

pub fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Camera>,
    used_screen_space: Res<OccupiedScreenSpace>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let size = UVec2::new(
            window.physical_width()
                - used_screen_space.left as u32
                - used_screen_space.right as u32,
            window.physical_height()
                - used_screen_space.top as u32
                - used_screen_space.bottom as u32,
        );

        for mut camera in &mut query {
            println!("Resize event with camera");
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(
                    used_screen_space.left as u32,
                    used_screen_space.top as u32,
                ),
                physical_size: size,
                ..default()
            });
        }
    }
}

fn interpolate_to_target(target: &mut Vec3, target_vec: Vec3, spring_mu: f32, delta_t: f32) {
    target.x = target.x - (target.x - target_vec.x) * spring_mu * delta_t;
    target.y = target.y - (target.y - target_vec.y) * spring_mu * delta_t;
    target.z = target.z - (target.z - target_vec.z) * spring_mu * delta_t;
}

fn interpolate_to_target_alt(
    target: &mut Vec3,
    target_vec: Vec3,
    follow_lag_ratio: f32,
    delta_t: f32,
) {
    target.x = target_vec.x * follow_lag_ratio;
    target.y = target_vec.y * follow_lag_ratio;
    target.z = target_vec.z * follow_lag_ratio;
}
