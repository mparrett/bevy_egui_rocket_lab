# Bevy Rocket Lab

A model rocketry simulator/game built with [Bevy](https://bevyengine.org/) 0.13. Early stage — currently a sandbox/tech demo.

Why? I recently revisited model rocketry with my children and found myself writing a python script to discern where our first launch might have landed. After that, I thought it might be fun to explore the idea a bit more.

## Quick Start

```bash
just run        # dev build + run
just fmt        # format code
just release    # optimized build
```

See [DEV.md](DEV.md) for module overview, WASM builds, and troubleshooting.

## TODO

- Minimum playable game (not sandbox mode)
  - Rocket building (currency, purchase body/cone/engines/fins/parachute/launch pads)
  - Objectives (landing, elevation, etc.) with constraints (wind, build deficiencies)
  - Title and game over screens
- Build and publish to web

## Ideas

- Better terrain ([blog post](https://clynamen.github.io/blog/2021/01/04/terrain_generation_bevy/), [warbler_grass](https://github.com/EmiOnGit/warbler_grass))
- Proper change detection via components
- Expand game states
- Camera upgrades (mouse pancam, FPS controller)

## Acknowledgements

This project should be distinct from [OpenRocket](https://openrocket.info/), which is pretty good at what it does. Maybe some future interoperability?

Based on [bevy-egui-playground](https://github.com/whoisryosuke/bevy-egui-playground) — thanks, whoisryosuke! Big thanks to my bro for inspiring me with his neat Bevy game jam entries ([itch.io](https://euclidean-whale.itch.io/)).

## References

- [nbody simulation](https://github.com/pjankiewicz/nbody/tree/master/src)
- [bevy_xpbd 3D chain example](https://github.com/Jondolf/bevy_xpbd/blob/main/crates/bevy_xpbd_3d/examples/chain_3d.rs)
- [bevy_firework pitch/yaw](https://github.com/mbrea-c/bevy_firework/blob/master/src/emission_shape.rs)
