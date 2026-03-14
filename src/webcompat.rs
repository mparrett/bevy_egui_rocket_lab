use bevy::prelude::*;

#[cfg(not(feature = "web_webgl"))]
use bevy::{post_process::bloom::Bloom, render::view::Hdr};

pub fn hdr_emissive(val: f32) -> f32 {
    #[cfg(feature = "web_webgl")]
    {
        val.min(1.0)
    }
    #[cfg(not(feature = "web_webgl"))]
    {
        val
    }
}

pub fn light_scale() -> f32 {
    #[cfg(feature = "web_webgl")]
    {
        // LDR framebuffer: scale down physically-based lux values so the
        // ground isn't blown out without tonemapping.
        0.45
    }
    #[cfg(not(feature = "web_webgl"))]
    {
        1.0
    }
}

pub fn skybox_brightness() -> f32 {
    #[cfg(feature = "web_webgl")]
    {
        // Without Hdr, the framebuffer is LDR. The cubemap textures store
        // values in [0,1] but are authored dim expecting a large HDR boost.
        // We use a moderate multiplier; values >1.0 clip but the dim sky
        // regions become visible. Combined with no tonemapping, this gives
        // a reasonable approximation of the HDR look.
        700.0
    }
    #[cfg(not(feature = "web_webgl"))]
    {
        1000.0
    }
}

pub fn insert_hdr_camera_components(entity: &mut EntityCommands) {
    #[cfg(not(feature = "web_webgl"))]
    {
        entity.insert((
            Hdr,
            bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
            Bloom {
                intensity: 0.1,
                high_pass_frequency: 0.35,
                ..Bloom::NATURAL
            },
        ));
    }
    #[cfg(feature = "web_webgl")]
    {
        entity.insert(bevy::core_pipeline::tonemapping::Tonemapping::None);
    }
}

pub fn insert_hdr_aux_camera_components(entity: &mut EntityCommands, bloom_intensity: f32) {
    #[cfg(not(feature = "web_webgl"))]
    {
        entity.insert((
            Hdr,
            bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
            Bloom {
                intensity: bloom_intensity,
                ..Bloom::NATURAL
            },
        ));
    }
    #[cfg(feature = "web_webgl")]
    {
        let _ = bloom_intensity;
        entity.insert(bevy::core_pipeline::tonemapping::Tonemapping::None);
    }
}
