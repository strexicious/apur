use winit::window::Window;

#[derive(Clone, Copy)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

struct Mesh {
    vertices: Vec<Vertex>,
    vbuffer: wgpu::Buffer,
}

impl Mesh {
    fn new(device: &wgpu::Device, vertices: Vec<Vertex>) -> Self {
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

struct Renderer {
    meshes: Vec<Mesh>,
    binds_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    fn new(device: &wgpu::Device) -> Self {
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

    fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    // TODO: create_binds is vague, and the appropiate funtions should be created,
    // like set light color, position, set fog thickness, etc.
    fn create_binds(&self, device: &wgpu::Device) -> wgpu::BindGroup {
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

    fn render(&self, frame: &wgpu::SwapChainOutput, cmd_encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device) {
        
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

pub struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    renderer: Renderer,
}

impl Engine {
    pub fn new(window: &Window) -> Self {
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,
        }).expect("Couldn't get hardware adapter");
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions::default(),
            limits: wgpu::Limits::default(),
        });
        
        let surface = wgpu::Surface::create(window);
        let swapchain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Vsync,
        });

        let mut renderer = Renderer::new(&device);
        let triangle = Mesh::new(&device, vec![
            Vertex { pos: [ 0.0, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [ 0.5,  0.5, 0.0], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [-0.5,  0.5, 0.0], color: [0.0, 0.0, 1.0] },
        ]);
        renderer.add_mesh(triangle);

        Self { device, queue, swapchain, renderer }
    }

    pub fn render(&mut self) {
        let frame = self.swapchain.get_next_texture();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.renderer.render(&frame, &mut encoder, &self.device);
        self.queue.submit(&[encoder.finish()]);
    }
}
