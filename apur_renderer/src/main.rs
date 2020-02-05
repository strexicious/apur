use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
    dpi::LogicalSize,
};

mod engine;

use engine::{Renderer, Mesh, Vertex};

fn handle_window_event(event: WindowEvent, control_flow: &mut ControlFlow) {
    #[allow(clippy::single_match)]
    match event {
        WindowEvent::CloseRequested => {
            println!("Shutting down...");
            *control_flow = ControlFlow::Exit;
        },
        _ => { }
    }
}

fn main() {
    
    const WIDTH: u16 = 800;
    const HEIGHT: u16 = 600;
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("APUR!")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .with_resizable(false)
        .build(&event_loop)
        .expect("Error building window");
    
    let surface = wgpu::Surface::create(&window);
    let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        backends: wgpu::BackendBit::PRIMARY,
    }).expect("Couldn't get hardware adapter");

    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions::default(),
        limits: wgpu::Limits::default(),
    });
    
    let mut swapchain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: WIDTH as u32,
        height: HEIGHT as u32,
        present_mode: wgpu::PresentMode::Vsync,
    });
    
    let mut renderer = Renderer::new(&device);
    let triangle = Mesh::new(&device, vec![
        Vertex { pos: ( 0.0, -0.5, 0.0), color: (1.0, 0.0, 0.0) },
        Vertex { pos: ( 0.5,  0.5, 0.0), color: (0.0, 1.0, 0.0) },
        Vertex { pos: (-0.5,  0.5, 0.0), color: (0.0, 0.0, 1.0) },
    ]);
    renderer.add_mesh(triangle);
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, ..} => handle_window_event(event, control_flow),
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let frame = swapchain.get_next_texture();
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
                renderer.render(&frame, &mut encoder, &device);
                queue.submit(&[encoder.finish()]);
            },
            _ => { }
        }
    });
}
