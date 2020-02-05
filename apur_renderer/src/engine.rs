#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub pos: (f32, f32, f32),
    pub color: (f32, f32, f32),
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    vbuffer: wgpu::Buffer,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: Vec<Vertex>) -> Self {
        let vbuffer = device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);
        Self { vertices, vbuffer }
    }

    fn draw(&self, pass: &mut wgpu::RenderPass) {
        pass.set_vertex_buffers(0, &[(&self.vbuffer, 0)]);
        pass.draw(0 .. self.vertices.len() as u32, 0 .. 1);
    }
}

pub struct Renderer {
    meshes: Vec<Mesh>,
    binds_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        // shaders
        let vs_source = include_bytes!("../res/shader.vert.spv");
        let vs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&vs_source[..])).expect("failed to read vertex shader spir-v"));

        let fs_source = include_bytes!("../res/shader.frag.spv");
        let fs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&fs_source[..])).expect("failed to read fragment shader spir-v"));

        // describe uniforms and storage blocks
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                    },
                }
            ]
        });

        // prepare and build the renderer pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
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
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: 24,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        offset: 12,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 1,
                    },
                ]
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            meshes: vec![],
            binds_layout: bind_group_layout,
            pipeline,
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    // TODO: create_binds is vague, and the appropiate funtions should be created,
    // like set light color, position, set fog thickness, etc.
    pub fn create_binds(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let props_buffer = device
            .create_buffer_mapped(3, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[0.3f32, -0.3f32, 0.0f32]);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.binds_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &props_buffer,
                        range: 0 .. 12,
                    },
                }
            ],
        })
    }

    pub fn render(&self, frame: &wgpu::SwapChainOutput, cmd_encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device) {
        
        const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.4, g: 0.1, b: 0.1, a: 1.0 };
        
        let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: CLEAR_COLOR,
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.create_binds(device), &[]);

        for mesh in &self.meshes {
            mesh.draw(&mut rpass);
        }
    }
}
