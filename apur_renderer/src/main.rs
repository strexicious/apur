use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder},
    dpi::LogicalSize,
};

mod engine;

use engine::{Engine};

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
    
    let mut ngn = Engine::new(&window);
        
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, ..} => handle_window_event(event, control_flow),
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => { ngn.render() },
            _ => { }
        }
    });
}
