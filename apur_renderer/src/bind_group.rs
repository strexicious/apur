use super::buffer::ManagedBuffer;
use super::texture::{Sampler, Texture};

#[derive(Debug)]
pub struct BindGroupBuilder<'a> {
    next_binding: usize,
    layout: &'a wgpu::BindGroupLayout,
    label: Option<&'a str>,
    bindings: Vec<wgpu::Binding<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn from_layout(layout: &'a wgpu::BindGroupLayout) -> Self {
        let next_binding = 0;
        let label = None;
        let bindings = vec![];

        Self {
            next_binding,
            layout,
            label,
            bindings,
        }
    }

    pub fn with_tag(mut self, tag: &'a str) -> Self {
        self.label = Some(tag);
        self
    }

    pub fn with_buffer(mut self, buffer: &'a ManagedBuffer) -> Self {
        self.bindings.push(wgpu::Binding {
            binding: self.next_binding as u32,
            resource: wgpu::BindingResource::Buffer {
                buffer: buffer.as_ref(),
                range: 0..buffer.size_bytes() as u64,
            },
        });

        self.next_binding += 1;
        self
    }

    pub fn with_sampler(mut self, sampler: &'a Sampler) -> Self {
        self.bindings.push(wgpu::Binding {
            binding: self.next_binding as u32,
            resource: wgpu::BindingResource::Sampler(sampler.as_ref()),
        });

        self.next_binding += 1;
        self
    }

    pub fn with_texture(mut self, texture: &'a Texture) -> Self {
        self.bindings.push(wgpu::Binding {
            binding: self.next_binding as u32,
            resource: wgpu::BindingResource::TextureView(texture.view()),
        });

        self.next_binding += 1;
        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: self.layout,
            bindings: &self.bindings,
            label: self.label,
        })
    }
}
