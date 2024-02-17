## TODO

- Minimum playable version (not sandbox mode)
- Build and publish


## IDEAS

- Title screen
- Game over screen
- Currency
- Purchase rocket parts: body, cone, engines, fins, launch pads(?)
- Better terrain:
  - https://clynamen.github.io/blog/2021/01/04/terrain_generation_bevy/
  - https://github.com/EmiOnGit/warbler_grass

// Use components and proper change detection
// https://github.com/bevyengine/bevy/blob/main/examples/ecs/component_change_detection.rs
// Game States
// https://bevy-cheatbook.github.io/programming/states.html
// Use 3p pancam? fps controller?
// See: https://github.com/pjankiewicz/nbody/tree/master/src
// Kinda cool example:
// https://github.com/Jondolf/bevy_xpbd/blob/main/crates/bevy_xpbd_3d/examples/chain_3d.rs
// Interesting example PitchYaw:
// https://github.com/mbrea-c/bevy_firework/blob/master/src/emission_shape.rs

// NOTE: Forked from bevy-egui-playground

## Development Notes

1. Wasm32 target

```
  = note: the `wasm32-unknown-unknown` target may not be installed
```

```
rustup target add wasm32-unknown-unknown
```

2. Wasm bindgen

```
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/rocket.wasm
sh: wasm-bindgen: command not found
```

```
cargo install wasm-bindgen-cli
```

Cargo clean does a lot.

```
❯ cargo clean
     Removed 33990 files, 25.2GiB total
```

/**
 * Frustrum code, unused at current
 *     // Frustum and window calculations
    /*
    let distance_to_target = (target - original_camera_transform.translation).length();
    let frustum_height = 2.0 * distance_to_target * (camera_projection.fov * 0.5).tan();
    let frustum_width = frustum_height * camera_projection.aspect_ratio;

    let window = windows.single();

    let left_taken = occupied_screen_space.left / window.width();
    let right_taken = occupied_screen_space.right / window.width();
    let top_taken = occupied_screen_space.top / window.height();
    let bottom_taken = occupied_screen_space.bottom / window.height();

    // Adjust camera position based on screen space
    let translation: Vec3 = original_camera_transform.translation
        + transform.rotation.mul_vec3(Vec3::new(
            (right_taken - left_taken) * frustum_width * 0.5,
            (top_taken - bottom_taken) * frustum_height * 0.5,
            0.0,
        ));
     */
 */
