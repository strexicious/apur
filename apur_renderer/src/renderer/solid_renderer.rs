use crate::pipeline::RenderPipeline;
use crate::shader::{SolidShader};
use crate::world::object::Object;
use crate::world::material::{Material, SolidMaterial};
use crate::buffer::ManagedBuffer;

pub struct SolidRenderer {
    pipeline: RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl SolidRenderer {
    pub fn new(device: &wgpu::Device, transforms_buffer: &ManagedBuffer, lights_buffer: &ManagedBuffer) -> Self {        
        let pipeline = RenderPipeline::new::<SolidShader>(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: pipeline.get_global_layout(),
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: transforms_buffer.get_buffer(),
                        range: 0 .. transforms_buffer.get_size_bytes() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: lights_buffer.get_buffer(),
                        range: 0 .. lights_buffer.get_size_bytes() as u64,
                    },
                },
            ],
            label: Some("SolidRenderer bind_group")
        });
        
        Self {
            bind_group,
            pipeline,
        }
    }

    pub fn generate_material(&self, device: &wgpu::Device, color: [f32; 3], roughness: f32) -> SolidMaterial {
        SolidMaterial::new(device, self.pipeline.get_element_layout(), color, roughness)
    }

    pub fn render<'a>(&'a mut self, rpass: &mut wgpu::RenderPass<'a>, objects: &'a [Object<SolidMaterial>]) {
        rpass.set_pipeline(self.pipeline.get_pipeline());
        rpass.set_bind_group(0, &self.bind_group, &[]);

        for obj in objects {
            let m = obj.get_mesh();
            let vbuf = m.get_vertex_buffer();
            let ibuf = m.get_index_buffer();
            
            rpass.set_vertex_buffer(0, vbuf.get_buffer(), 0, vbuf.get_size_bytes() as u64);
            rpass.set_index_buffer(ibuf.get_buffer(), 0, ibuf.get_size_bytes() as u64);

            let mat_bg = obj.get_material().get_bind_group();
            rpass.set_bind_group(1, mat_bg, &[]);
            rpass.draw_indexed(0..m.get_index_count() as u32, 0, 0..1);
        }
    }
}