pub mod camera;
pub mod light;
pub mod object;
pub mod material;

use camera::Camera;
use light::Light;
use object::Object;
use material::SolidMaterial;

pub struct World {
    solid_objects: Vec<Object<SolidMaterial>>,
    lights: Vec<Light>,
    camera: Camera,
}

impl World {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            solid_objects: vec![],
            lights: vec![],
            camera: Camera::new(width, height),
        }
    }
}
