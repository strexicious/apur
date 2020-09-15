use super::buffer::ManagedBuffer;
use super::error as apur_error;

#[derive(Debug)]
pub struct BindGroupLayoutBuilder<'a> {
    label: Option<&'a str>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'a> BindGroupLayoutBuilder<'a> {
    pub fn new() -> Self {
        let label = None;
        let entries = vec![];

        Self { label, entries }
    }

    pub fn with_tag(mut self, tag: &'a str) -> Self {
        self.label = Some(tag);
        self
    }

    pub fn with_binding(
        mut self,
        visibility: wgpu::ShaderStage,
        binding_type: wgpu::BindingType,
    ) -> Self {
        let binding = self.entries.len();
        self.entries.push(wgpu::BindGroupLayoutEntry {
            visibility,
            binding: binding as u32,
            ty: binding_type,
            count: None,
        });
        self
    }

    pub fn build(self, device: &wgpu::Device) -> BindGroupLayout {
        let entries = self.entries;
        let inner_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: None,
        });

        BindGroupLayout {
            entries,
            inner_layout,
        }
    }
}

#[derive(Debug)]
pub struct BindGroupLayout {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
    inner_layout: wgpu::BindGroupLayout,
}

impl BindGroupLayout {
    pub fn to_bind_group_builder(&self) -> BindGroupBuilder {
        let layout = self;
        let label = None;
        let bindings = vec![];

        BindGroupBuilder {
            layout,
            label,
            bindings,
        }
    }
}

impl AsRef<wgpu::BindGroupLayout> for BindGroupLayout {
    fn as_ref(&self) -> &wgpu::BindGroupLayout {
        &self.inner_layout
    }
}

#[derive(Debug)]
pub struct BindGroupBuilder<'a> {
    layout: &'a BindGroupLayout,
    label: Option<&'a str>,
    bindings: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn with_tag(mut self, tag: &'a str) -> Self {
        self.label = Some(tag);
        self
    }

    pub fn with_buffer(mut self, buffer: &'a ManagedBuffer) -> apur_error::Result<Self> {
        if self.bindings.len() == self.layout.entries.len() {
            return Err(apur_error::APURRendererError::NumOfBindingsOverflowed);
        }

        let binding = self.bindings.len();
        match self.layout.entries[binding as usize].ty {
            wgpu::BindingType::UniformBuffer { .. } | wgpu::BindingType::StorageBuffer { .. } => {}
            _ => return Err(apur_error::APURRendererError::BindingResourceTypeUnmatched),
        }

        self.bindings.push(wgpu::BindGroupEntry {
            binding: binding as u32,
            resource: wgpu::BindingResource::Buffer(buffer.as_ref().slice(..)),
        });
        Ok(self)
    }

    pub fn with_sampler(mut self, sampler: &'a wgpu::Sampler) -> apur_error::Result<Self> {
        if self.bindings.len() == self.layout.entries.len() {
            return Err(apur_error::APURRendererError::NumOfBindingsOverflowed);
        }

        let binding = self.bindings.len();
        match self.layout.entries[binding as usize].ty {
            wgpu::BindingType::Sampler { .. } => {}
            _ => return Err(apur_error::APURRendererError::BindingResourceTypeUnmatched),
        }

        self.bindings.push(wgpu::BindGroupEntry {
            binding: binding as u32,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
        Ok(self)
    }

    pub fn with_texture(mut self, texture_view: &'a wgpu::TextureView) -> apur_error::Result<Self> {
        if self.bindings.len() == self.layout.entries.len() {
            return Err(apur_error::APURRendererError::NumOfBindingsOverflowed);
        }

        let binding = self.bindings.len();
        match self.layout.entries[binding as usize].ty {
            wgpu::BindingType::SampledTexture { .. } => {}
            _ => return Err(apur_error::APURRendererError::BindingResourceTypeUnmatched),
        }

        self.bindings.push(wgpu::BindGroupEntry {
            binding: binding as u32,
            resource: wgpu::BindingResource::TextureView(texture_view),
        });
        Ok(self)
    }

    pub fn build(self, device: &wgpu::Device) -> apur_error::Result<wgpu::BindGroup> {
        if self.bindings.len() < self.layout.entries.len() {
            return Err(apur_error::APURRendererError::NumOfBindingsUnderflowed);
        }

        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: self.layout.as_ref(),
            entries: &self.bindings,
            label: self.label,
        }))
    }
}
