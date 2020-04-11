use glam::{Vec3};

#[derive(Default, Debug)]
pub struct DirectionalLight {
    pos: Vec3,
    direction: Vec3,
    color: Vec3,
}

pub enum Light {
    Directional(DirectionalLight),
}
