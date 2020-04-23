use std::time::{Duration, Instant};

use winit::{
    event::{Event, WindowEvent, DeviceEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
    dpi::LogicalSize,
};

use apur_renderer::engine::Engine;
use apur_renderer::world::World;

fn main() {
    env_logger::init();
    
    const WIDTH: u16 = 800;
    const HEIGHT: u16 = 600;
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("APUR!")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .with_resizable(false)
        .build(&event_loop)
        .expect("Error building window");
    window.set_cursor_visible(false);
    // window.set_cursor_grab(true).expect("Couldn't lock the cursor...");
    
    let world = World::new(WIDTH as u32, HEIGHT as u32);
    let mut ngn = Engine::new(&window, world);
    let mut close_request = false;
    let mut last_tick = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let cur_tick = Instant::now();

        match event {
            // Event::WindowEvent { event, ..} => handle_window_event(&mut ngn, event, &mut close_request, cur_tick - last_tick),
            Event::MainEventsCleared => {
                if close_request {
                    println!("Shutting down...");
                    *control_flow = ControlFlow::Exit;
                } else {
                    window.request_redraw();
                }
            },
            Event::RedrawRequested(_) => { ngn.render() },
            // Event::DeviceEvent { event, .. } => handle_device_event(&mut ngn, event),
            _ => { }
        }
        last_tick = cur_tick;
    });
}
