use super::bind_group::BindGroupLayout;
use super::error::{self, APURRendererError};
use std::fs::File;
use std::path::Path;

pub trait RenderShader {
    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static>;
    const COLOR_STATE_DESCS: &'static [wgpu::ColorStateDescriptor];
    const DEPTH_STENCIL_DESC: Option<wgpu::DepthStencilStateDescriptor>;

    fn layouts(&self) -> &[BindGroupLayout];
    fn vertex_module_path(&self) -> &Path;
    fn fragment_module_path(&self) -> &Path;
}

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {
    pub fn new<S: RenderShader>(device: &wgpu::Device, shader: &S) -> error::Result<Self> {
        let layouts = shader
            .layouts()
            .iter()
            .map(|l| l.as_ref())
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: layouts.as_slice(),
        });

        let vmodule = device.create_shader_module(
            &wgpu::read_spirv(
                File::open(shader.vertex_module_path())
                    .map_err(|_| APURRendererError::ErrorOpeningShaderSPV)?,
            )
            .expect("failed to create vertex shader spir-v"),
        );

        let fmodule = device.create_shader_module(
            &wgpu::read_spirv(
                File::open(shader.fragment_module_path())
                    .map_err(|_| APURRendererError::ErrorOpeningShaderSPV)?,
            )
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
            color_states: S::COLOR_STATE_DESCS,
            depth_stencil_state: S::DEPTH_STENCIL_DESC,
            vertex_state: S::VERTEX_STATE_DESC,
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Ok(Self { pipeline })
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
