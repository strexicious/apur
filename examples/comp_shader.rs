use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::{BindGroupLayout, BindGroupLayoutBuilder},
    buffer::ManagedBuffer,
    error as apur_error,
    event_handler::EventHandler,
    pipeline::{ComputePipeline, ComputeShader},
};
use futures::{executor, FutureExt};
use std::path::Path;

const WIDTH: u16 = 800;
const HEIGHT: u16 = 800;

struct CollatzShader {
    layouts: Vec<BindGroupLayout>,
}

impl CollatzShader {
    fn new(device: &wgpu::Device) -> Self {
        let layout = BindGroupLayoutBuilder::new()
            .with_binding(
                wgpu::ShaderStage::COMPUTE,
                wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: false,
                    min_binding_size: None,
                },
            )
            .build(device);
        let layouts = vec![layout];

        Self { layouts }
    }
}

impl ComputeShader for CollatzShader {
    fn layouts(&self) -> &[BindGroupLayout] {
        &self.layouts
    }

    fn compute_module_path(&self) -> &Path {
        "res/shaders/collatz/collatz.comp.spv".as_ref()
    }
}

struct GeneralDriver {
    pipe: ComputePipeline,
    nums: ManagedBuffer,
    bind_group: wgpu::BindGroup,
    done: bool,
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> apur_error::Result<Self> {
        let nums = ManagedBuffer::from_data::<u32>(
            device,
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::MAP_READ,
            &[1, 4, 3, 295],
        );

        let shader = CollatzShader::new(device);
        let bind_group = shader.layouts()[0]
            .to_bind_group_builder()
            .with_buffer(&nums)?
            .build(device)?;

        let pipe = ComputePipeline::new(device, shader)?;

        let done = false;

        Ok(Self {
            pipe,
            nums,
            bind_group,
            done,
        })
    }
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        None
    }

    fn update(&mut self, app: &mut Application) {
        if !self.done {
            let mut encoder =
                app.device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("compute encoder"),
                    });

            {
                let mut cpass = encoder.begin_compute_pass();
                cpass.set_pipeline(self.pipe.as_ref());
                cpass.set_bind_group(0, &self.bind_group, &[]);
                cpass.dispatch(4, 1, 1);
            }

            let queue = app.queue();
            queue.submit(vec![encoder.finish()])
        }
    }

    fn render(&mut self, app: &mut Application, frame: &wgpu::SwapChainFrame) {
        if !self.done {
            let output = executor::block_on(apur::future::post_pending(
                self.nums.read_data::<u32>().boxed(),
                || app.device().poll(wgpu::Maintain::Wait),
            ));

            println!("{:?}", output.unwrap().as_slice());

            self.done = true;
        }
    }
}

fn main() {
    env_logger::init();

    let app = executor::block_on(Application::new("solid-shader example", WIDTH, HEIGHT)).unwrap();

    match GeneralDriver::new(app.device()) {
        Ok(driver) => app.run(driver),
        Err(e) => eprintln!("startup error: {:?}", e),
    }
}
