use bevy::prelude::*;

fn rand_sign() -> f32{
    let r = fastrand::f32();
    2. * if r > 0.5 {  r - 1. } else { r} 
}

pub fn random_pos(base: Vec3, quant: f32) -> Vec3 {
    base + Vec3::Z * rand_sign() * quant + Vec3::X * rand_sign() * quant
} 





