use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use wgpu::{Surface, Adapter, Color};

const WINDOW_WIDTH: u16 = 800;
const WINDOW_HEIGHT: u16 = 600;

fn main() {
    let el = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("APUR!")
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_resizable(false)
        .build(&el)
        .expect("Error building window");
    
    let surface = Surface::create(&window);
    let adapter = Adapter::request(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        backends: wgpu::BackendBit::PRIMARY,
    }).expect("Couldn't get hardware adapter");

    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions::default(),
        limits: wgpu::Limits::default(),
    });
    
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

    let props_buffer = device
        .create_buffer_mapped(3, wgpu::BufferUsage::UNIFORM)
        .fill_from_slice(&[0.3f32, -0.3f32, 0.0f32]);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &props_buffer,
                    range: 0 .. 12,
                },
            }
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let vs = include_bytes!("../res/shader.vert.spv");
    let vs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());

    let fs = include_bytes!("../res/shader.frag.spv");
    let fs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());
    
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
    
    let clear_color = Color { r: 0.4, g: 0.1, b: 0.1, a: 1.0, };
    let data: [f32; 18] = [
        // position    color    
         0.0, -0.5, 0.0, 1.0, 0.0, 0.0,
         0.5,  0.5, 0.0, 0.0, 1.0, 0.0,
        -0.5,  0.5, 0.0, 0.0, 0.0, 1.0,
    ];
    let tri_vbuffer = device
        .create_buffer_mapped(18, wgpu::BufferUsage::VERTEX)
        .fill_from_slice(&data);

    let sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: WINDOW_WIDTH as u32,
        height: WINDOW_HEIGHT as u32,
        present_mode: wgpu::PresentMode::Vsync,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, ..} => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("Shutting down...");
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => { }
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                let frame = swap_chain.get_next_texture();
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color,
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_vertex_buffers(0, &[(&tri_vbuffer, 0)]);
                    rpass.draw(0 .. 3, 0 .. 1);
                }

                queue.submit(&[encoder.finish()]);
            },
            _ => { }
        }
    });

}
