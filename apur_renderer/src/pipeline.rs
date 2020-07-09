/// Well we could make Shader a dynamic object,
/// but for now lets just say that shaders will
/// be created at compile time, and these constants
/// in the traits are like a "constant functions"
/// See solid_shader example for usage.
pub trait RenderShader {
    const GLOBAL_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static>;
    const ELEMENT_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static>;
    const VERTEX_MODULE: &'static [u8];
    const FRAGMENT_MODULE: &'static [u8];
    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static>;
}

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    global_layout: wgpu::BindGroupLayout,
    element_layout: wgpu::BindGroupLayout,
}

impl RenderPipeline {
    pub fn new<T: RenderShader>(device: &wgpu::Device) -> Self {
        let global_layout = device.create_bind_group_layout(&T::GLOBAL_LAYOUT_DESC);
        let element_layout = device.create_bind_group_layout(&T::ELEMENT_LAYOUT_DESC);
    
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&global_layout, &element_layout],
        });

        let vmodule = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(T::VERTEX_MODULE)).expect("failed to create vertex shader spir-v"));
    
        let fmodule = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(T::FRAGMENT_MODULE)).expect("failed to create fragment shader spir-v"));
    
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vmodule,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fmodule,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil_front: wgpu::StencilStateFaceDescriptor::default(),
                stencil_back: wgpu::StencilStateFaceDescriptor::default(),
                stencil_read_mask: !0,
                stencil_write_mask: !0,
            }),
            vertex_state: T::VERTEX_STATE_DESC.clone(),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            pipeline,
            global_layout,
            element_layout,
        }
    }

    pub fn global_layout(&self) -> &wgpu::BindGroupLayout {
        &self.global_layout
    }

    pub fn element_layout(&self) -> &wgpu::BindGroupLayout {
        &self.element_layout
    }
}

impl AsRef<wgpu::RenderPipeline> for RenderPipeline {
    fn as_ref(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
