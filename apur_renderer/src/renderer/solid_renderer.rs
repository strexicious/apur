use crate::pipeline::RenderPipeline;
use crate::shader::{SolidShader};
use crate::world::object::Object;
use crate::world::material::SolidMaterial;

pub struct SolidRenderer {
    pipeline: RenderPipeline,
}

impl SolidRenderer {
    pub fn new(device: &wgpu::Device) -> Self {        
        Self { pipeline: RenderPipeline::new::<SolidShader>(device) }
    }

    pub fn render(&mut self, objects: Vec<Object<SolidMaterial>>) {
        // self.pipeline.bind();
        // for obj in objects {
        //     obj.draw();
        // }
    }
}