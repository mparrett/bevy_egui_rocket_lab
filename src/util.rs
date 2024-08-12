use bevy::math::Vec3;

pub fn random_vec(magnitude: f32, proba: f32) -> Vec3 {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    if rand::random::<f32>() < proba {
        x = magnitude;
    }
    if rand::random::<f32>() < proba {
        y = magnitude;
    }
    if rand::random::<f32>() < proba {
        z = magnitude;
    }
    if rand::random::<f32>() < 0.5 {
        x = -x;
    }
    if rand::random::<f32>() < 0.5 {
        y = -y;
    }
    if rand::random::<f32>() < 0.5 {
        z = -z;
    }
    Vec3::new(x, y, z)
}
