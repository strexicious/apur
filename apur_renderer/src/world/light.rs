use glam::{Vec3};

#[derive(Debug)]
pub enum Light {
    Directional {
        direction: Vec3,
        color: Vec3,
    },
}

impl Light {
    pub fn to_shader_data(&self) -> Vec<f32> {
        let mut res = vec![];
        match self {
            Light::Directional { direction, color } => {
                res.extend(direction.as_ref());
                res.push(0.0); // padding for how uniform alignment works :/
                res.extend(color.as_ref());
            }
        }
        res
    }
}
