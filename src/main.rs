use std::path::Path;
use glutin::dpi::*;
use glutin::ContextTrait;

mod shader;
use shader::{UnlinkedProgram, Program};

fn main() {
    let mut el = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new()
        .with_title("APUR!")
        .with_dimensions(LogicalSize::new(800.0, 600.0));
    let windowed_context = glutin::ContextBuilder::new()
        .build_windowed(wb, &el)
        .unwrap();
    
    unsafe {
        windowed_context.make_current().unwrap();
    }

    unsafe {
        gl::load_with(|symbol| windowed_context.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.4, 0.1, 0.1, 1.0);
    }

    // data
    let triangle_vertices: [f32; 9] = [
        0.0, 0.5, 0.0,
        0.5,-0.5, 0.0,
       -0.5,-0.5, 0.0
    ];

    // intitialization
    let mut vao: gl::types::GLuint = 0;
    let mut vbo: gl::types::GLuint = 0;
    unsafe {
        // buffer object
        gl::GenBuffers(1, &mut vbo as *mut gl::types::GLuint);
        
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            9,
            triangle_vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        // vertex array object
        gl::GenVertexArrays(1, &mut vao as *mut gl::types::GLuint);
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            0,
            std::ptr::null()
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    };

    // for shaders
    let mut unlinked_program = UnlinkedProgram::new();
    unlinked_program.add_shader(Path::new("res/shader.vert"), gl::VERTEX_SHADER).unwrap();
    unlinked_program.add_shader(Path::new("res/shader.frag"), gl::FRAGMENT_SHADER).unwrap();
    let program = unlinked_program.link().unwrap_or_else(move |error| {
        eprintln!("An error occured on link: {}", error);
        ::std::process::exit(-1);
    });

    // update -- display
    let mut running = true;
    while running {
        el.poll_events(|event| {
            if let glutin::Event::WindowEvent{ event, .. } =  event {
                match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        let dpi_factor = windowed_context.get_hidpi_factor();
                        windowed_context.resize(logical_size.to_physical(dpi_factor));
                    },
                    _ => ()
                }
            }
        });

        program.activate();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                3
            );
        }

        windowed_context.swap_buffers().unwrap();
    }

    Program::deactivate();
}
