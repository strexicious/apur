use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
    dpi::LogicalSize,
};

mod engine;

use engine::{Engine};

fn handle_window_event(event: WindowEvent, close_request: &mut bool) {
    #[allow(clippy::single_match)]
    match event {
        WindowEvent::CloseRequested => *close_request = true,
        WindowEvent::KeyboardInput { input, .. } => {
            match input.scancode {
                // escape key
                0x01 => *close_request = true,
                _ => { },
            }
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
    window.set_cursor_visible(false);
    window.set_cursor_grab(true).expect("Couldn't lock the cursor...");
    
    let mut ngn = Engine::new(&window);
    let mut close_request = false;
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, ..} => handle_window_event(event, &mut close_request),
            Event::MainEventsCleared => {
                if close_request {
                    println!("Shutting down...");
                    *control_flow = ControlFlow::Exit;
                } else {
                    window.request_redraw();
                }
            },
            Event::RedrawRequested(_) => { ngn.render() },
            Event::DeviceEvent { event, .. } => ngn.handle_device_event(event),
            _ => { }
        }
    });
}
