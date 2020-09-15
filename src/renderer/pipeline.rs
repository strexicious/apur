use std::path::Path;

use super::bind_group::BindGroupLayout;
use super::error::{self, APURRendererError};

pub trait RenderShader {
    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static>;
    const COLOR_STATE_DESCS: &'static [wgpu::ColorStateDescriptor];
    const DEPTH_STENCIL_DESC: Option<wgpu::DepthStencilStateDescriptor>;

    fn layouts(&self) -> &[BindGroupLayout];
    fn vertex_module_path(&self) -> &Path;
    fn fragment_module_path(&self) -> Option<&Path>;
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
            label: None,
            bind_group_layouts: layouts.as_slice(),
            push_constant_ranges: &[],
        });

        let vmodule = std::fs::read(shader.vertex_module_path())
            .map(|data| device.create_shader_module(wgpu::util::make_spirv(&data)))
            .map_err(|_| APURRendererError::ErrorOpeningShaderSPV)?;

        let fmodule = shader
            .fragment_module_path()
            .map(|frag_path| {
                std::fs::read(frag_path)
                    .map(|data| device.create_shader_module(wgpu::util::make_spirv(&data)))
                    .map_err(|_| APURRendererError::ErrorOpeningShaderSPV)
            })
            .transpose()?;

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vmodule,
                entry_point: "main",
            },
            fragment_stage: fmodule
                .as_ref()
                .map(|module| wgpu::ProgrammableStageDescriptor {
                    module: module,
                    entry_point: "main",
                }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: S::COLOR_STATE_DESCS,
            depth_stencil_state: S::DEPTH_STENCIL_DESC,
            vertex_state: S::VERTEX_STATE_DESC,
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
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
    fn compute_module_path(&self) -> &Path;
}

pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
}

impl ComputePipeline {
    pub fn new(device: &wgpu::Device, shader: impl ComputeShader) -> error::Result<Self> {
        let layouts = shader
            .layouts()
            .iter()
            .map(|l| l.as_ref())
            .collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: layouts.as_slice(),
            push_constant_ranges: &[],
        });

        let cmodule = std::fs::read(shader.compute_module_path())
            .map(|data| device.create_shader_module(wgpu::util::make_spirv(&data)))
            .map_err(|_| APURRendererError::ErrorOpeningShaderSPV)?;

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: Some(&pipeline_layout),
            compute_stage: {
                wgpu::ProgrammableStageDescriptor {
                    module: &cmodule,
                    entry_point: "main",
                }
            },
            label: None,
        });

        Ok(Self { pipeline })
    }
}

impl AsRef<wgpu::ComputePipeline> for ComputePipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
