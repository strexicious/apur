use super::shader::Shader;

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    global_layout: wgpu::BindGroupLayout,
    element_layout: wgpu::BindGroupLayout,
}

impl RenderPipeline {
    pub fn new<T: Shader>(device: &wgpu::Device) -> Self {
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
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
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

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn get_global_layout(&self) -> &wgpu::BindGroupLayout {
        &self.global_layout
    }

    pub fn get_element_layout(&self) -> &wgpu::BindGroupLayout {
        &self.element_layout
    }
}
