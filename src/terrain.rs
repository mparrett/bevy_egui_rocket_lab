use bevy_generative::noise::Gradient;
use bevy_generative::noise::Noise;
use bevy_generative::noise::Region;
use bevy_generative::terrain::Terrain;

fn spawn_terrain(
    material_handle: Handle<StandardMaterial>,
    x_offset: f32,
    z_offset: f32,
) -> TerrainBundle {
    let desired_terrain_size = GROUND_SIZE * 2.;
    let terrain_size = 5;
    let scale_factor = desired_terrain_size / terrain_size as f32;
    let terrain_resolution = 8;
    let mut noise = Noise::default();
    noise.regions = vec![
        Region {
            label: "Region #1".to_string(),
            color: [255, 0, 0, 255],
            position: 0.0,
        },
        Region {
            label: "Region #2".to_string(),
            color: [0, 0, 255, 255],
            position: 100.0,
        },
    ];

    noise.gradient = Gradient {
        image: Handle::default(),
        size: [250, 50],
        segments: 0,
        smoothness: 0.0,
    };

    let mut transform = Transform::from_xyz(x_offset, 0.0, z_offset);
    if scale_factor != 1.0 {
        println!("Scaling terrain by {}", scale_factor);
        transform = transform.with_scale(Vec3::new(scale_factor, scale_factor, scale_factor));
    }

    TerrainBundle {
        terrain: Terrain {
            noise: noise,
            size: [terrain_size; 2],
            resolution: terrain_resolution,
            wireframe: false,
            height_exponent: 1.0,
            sea_percent: 14.0, // 8.0, // 50.0, //12.0,
            export: false,
        },
        pbr_bundle: PbrBundle {
            material: material_handle,
            transform: transform,
            ..Default::default()
        },
    }
}

fn do_spawn_terrain() {
    let terrain_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.8, 0.2),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    /*
    commands.spawn(
        spawn_terrain(terrain_material.clone(), 0.0, 0.0));
    */
    /*
    commands.spawn(
        spawn_terrain(terrain_material.clone(), 0.0, -GROUND_SIZE));
    commands.spawn(
        spawn_terrain(terrain_material.clone(), 0.0, GROUND_SIZE));
    commands.spawn(
        spawn_terrain(terrain_material.clone(), -GROUND_SIZE, 0.0));
    commands.spawn(
        spawn_terrain(terrain_material.clone(), GROUND_SIZE, 0.0));
    */
}
