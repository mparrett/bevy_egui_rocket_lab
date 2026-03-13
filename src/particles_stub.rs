use bevy::prelude::*;

#[derive(Resource)]
pub struct ParticleProperties {
    pub exhaust_lifetime: f32,
    pub active_smoke_lifetime: f32,
    pub residual_smoke_lifetime: f32,
}

impl Default for ParticleProperties {
    fn default() -> Self {
        Self {
            exhaust_lifetime: 0.8,
            active_smoke_lifetime: 6.5,
            residual_smoke_lifetime: 3.25,
        }
    }
}
