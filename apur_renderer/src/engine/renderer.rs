use glam::{Mat4};

use super::model::{Vertex, Model};

pub struct RenderData {
    model: Model,
    bind_group: wgpu::BindGroup,
    texture_binds: Vec<wgpu::BindGroup>,
    uniforms_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
}

impl RenderData {
    pub fn new(
        device: &wgpu::Device,
        model: Model,
        view_trans: Mat4,
        proj_trans: Mat4,
        bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let uniforms_buffer = device
            .create_buffer_mapped(2, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[view_trans, proj_trans]);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniforms_buffer,
                        range: 0 .. 2 * 64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ]
        });

        let texture_binds = model.get_meshes().iter().map(|m| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: texture_bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(m.get_texture_view()),
                    },
                ]
            })
        }).collect();

        Self { model, texture_binds, bind_group, uniforms_buffer, sampler }
    }
}

pub struct Renderer {
    bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vs_source = include_bytes!("../../res/shaders/shader.vert.spv");
        let vs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&vs_source[..])).expect("failed to read vertex shader spir-v"));

        let fs_source = include_bytes!("../../res/shaders/shader.frag.spv");
        let fs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&fs_source[..])).expect("failed to read fragment shader spir-v"));
            
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ]
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
            ]
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout, &texture_bind_group_layout],
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
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
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        offset: 12,
                        format: wgpu::VertexFormat::Float2,
                        shader_location: 1,
                    },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
            
        Self { bind_group_layout, texture_bind_group_layout, pipeline }
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn get_texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    pub fn render(
        &self,
        frame: &wgpu::SwapChainOutput,
        cmd_encoder: &mut wgpu::CommandEncoder,
        depth_texture_view: &wgpu::TextureView,
        render_data: &RenderData,
    ) {
        
        const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.4, g: 0.1, b: 0.1, a: 1.0 };
        
        let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: CLEAR_COLOR,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth_texture_view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &render_data.bind_group, &[]);
        rpass.set_vertex_buffers(0, &[(render_data.model.get_vertex_buffer(), 0)]);
        rpass.set_index_buffer(render_data.model.get_indices_buffer(), 0);

        let meshes = render_data.model.get_meshes().iter();
        for (mesh, texture_bind_group) in meshes.zip(render_data.texture_binds.iter()) {
            rpass.set_bind_group(1, texture_bind_group, &[]);
            rpass.draw_indexed(mesh.get_indices_offset()..mesh.get_indices_offset()+mesh.get_indices_count(), 0, 0..1);
        }
    }
}