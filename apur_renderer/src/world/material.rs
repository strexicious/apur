use crate::buffer::ManagedBuffer;

pub trait Material {
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

pub struct SolidMaterial {
    color: [f32; 3],
    roughness: f32,
    uniform_buffer: ManagedBuffer,
    bind_group: wgpu::BindGroup,
}

impl SolidMaterial {
    pub fn new(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, color: [f32; 3], roughness: f32) -> Self {
        let uniform_buffer = ManagedBuffer::from_data(device, wgpu::BufferUsage::UNIFORM, &[color[0], color[1], color[2], roughness]);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: uniform_buffer.get_buffer(),
                        range: 0..uniform_buffer.get_size_bytes() as u64,
                    },
                },
            ],
            label: None
        });

        Self {
            color,
            roughness,
            uniform_buffer,
            bind_group,
        }
    }
}

impl Material for SolidMaterial {
    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
