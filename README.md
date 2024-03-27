# Bevy Rocket Lab

A model rocketry game, tool, or simulator. Not sure yet.

Why? I recently revisited model rocketry with my children and found myself writing a python script to discern where our our first launch might have landed. After that, I thought it might be fun to explore the idea a bit more.

# Acknowledgements

This project should be distinct from [OpenRocket](https://openrocket.info/), which is pretty good at what it does. Maybe some future interoperability?

This project used [bevy-egui-playground](https://github.com/whoisryosuke/bevy-egui-playground) as a base. Thanks, whoisryosuke!

Big thanks to my bro for inspiring me with his neat Bevy game jam entries. Check those out at [itch.io](https://euclidean-whale.itch.io/).

## TODO

- Minimum playable game (not sandbox mode)
  - Rocket building
    - Currency
    - At minimum, purchase body, cone, engines, fins, parachute, launch pads(?)
  - Objectives such as landing, elevation, etc. with constraints
    - Wind
    - Minor build deficiencies (glued fins at slightly wrong angle, etc. can be improved with skill)
  - Title and game over screens
  
- Able to build and publish to web

## IDEAS

- Better terrain:
  - https://clynamen.github.io/blog/2021/01/04/terrain_generation_bevy/
  - https://github.com/EmiOnGit/warbler_grass

- Use components and proper change detection. https://github.com/bevyengine/bevy/blob/main/examples/ecs/component_change_detection.rs
- Utilize game states. https://bevy-cheatbook.github.io/programming/states.html
- Camera upgrades... Use 3p mouse pancam? fps controller?

## References and Inspiration

- https://github.com/pjankiewicz/nbody/tree/master/src
- https://github.com/Jondolf/bevy_xpbd/blob/main/crates/bevy_xpbd_3d/examples/chain_3d.rs
- Interesting example for pitch/yaw. https://github.com/mbrea-c/bevy_firework/blob/master/src/emission_shape.rs


## Development Notes / Troubleshooting

See [DEV.md](DEV.md) for more info.
