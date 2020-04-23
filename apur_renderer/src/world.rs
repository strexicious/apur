use glam::Vec3;

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
            lights: vec![
                Light::Directional {
                    direction: Vec3::new(0.0, -1.0, 0.0),
                    color: Vec3::one(),
                }
            ],
            camera: Camera::new(width, height),
        }
    }

    pub fn get_solid_objects(&self) -> &[Object<SolidMaterial>] {
        &self.solid_objects
    }

    pub fn get_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn get_lights(&self) -> &[Light] {
        &self.lights
    }
}
