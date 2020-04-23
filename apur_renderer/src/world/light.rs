use glam::{Vec3};

#[derive(Debug)]
pub enum Light {
    Directional {
        pos: Vec3,
        direction: Vec3,
        color: Vec3,
    },
}
