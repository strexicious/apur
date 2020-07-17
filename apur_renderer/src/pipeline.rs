use super::bind_group::BindGroupLayout;

pub trait RenderShader {
    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static>;

    fn layouts(&self) -> &[BindGroupLayout];
    fn vertex_module(&self) -> &[u8];
    fn fragment_module(&self) -> &[u8];
}

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn new<S: RenderShader>(device: &wgpu::Device, shader: S) -> Self {
        let layouts = shader
            .layouts()
            .iter()
            .map(|l| l.as_ref())
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: layouts.as_slice(),
        });

        let vmodule = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(shader.vertex_module()))
                .expect("failed to create vertex shader spir-v"),
        );

        let fmodule = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(shader.fragment_module()))
                .expect("failed to create fragment shader spir-v"),
        );

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
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
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
            vertex_state: S::VERTEX_STATE_DESC.clone(),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self { pipeline }
    }
}

impl AsRef<wgpu::RenderPipeline> for RenderPipeline {
    fn as_ref(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

pub trait ComputeShader {
    fn layouts(&self) -> &[BindGroupLayout];
    fn compute_module(&self) -> &[u8];
}

pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
}

impl ComputePipeline {
    pub fn new(device: &wgpu::Device, shader: impl ComputeShader) -> Self {
        let layouts = shader
            .layouts()
            .iter()
            .map(|l| l.as_ref())
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: layouts.as_slice(),
        });

        let cmodule = device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(shader.compute_module()))
                .expect("failed to create vertex shader spir-v"),
        );

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: {
                wgpu::ProgrammableStageDescriptor {
                    module: &cmodule,
                    entry_point: "main",
                }
            },
        });

        Self { pipeline }
    }
}

impl AsRef<wgpu::ComputePipeline> for ComputePipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
