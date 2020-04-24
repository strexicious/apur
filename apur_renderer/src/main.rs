use std::time::{Duration, Instant};

use winit::{
    event::{Event, WindowEvent, DeviceEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
    dpi::LogicalSize,
};

use apur_renderer::engine::Engine;

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
    
    let mut ngn = Engine::new(&window);
    let mut close_request = false;
    let mut last_tick = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let cur_tick = Instant::now();
        let spf = (cur_tick - last_tick).as_secs_f32();

        match event {
            Event::WindowEvent {event, ..} => {
                match event {
                    WindowEvent::CloseRequested => close_request = true,
                    WindowEvent::KeyboardInput { input, .. } => {
                        match input.scancode {
                            // escape key
                            0x01 => close_request = true,
                            0x21 => println!("FPS: {}", 1.0 / spf),
                            _ => {},
                        }
                        ngn.handle_key_input(input);
                    },
                    _ => {}
                }
            },
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion{ delta: (dx, dy) } = event {
                    ngn.handle_mouse_move(dx, dy);
                }
            },
            Event::MainEventsCleared => {
                if close_request {
                    println!("Shutting down...");
                    *control_flow = ControlFlow::Exit;
                } else {
                    window.request_redraw();
                }
            },
            Event::RedrawRequested(_) => { ngn.render() },
            _ => { }
        }
        last_tick = cur_tick;
    });
}
