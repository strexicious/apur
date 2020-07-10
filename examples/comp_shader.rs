use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::BindGroupBuilder,
    buffer::ManagedBuffer,
    event_handler::EventHandler,
    pipeline::{ComputePipeline, ComputeShader},
};
use futures::{executor, FutureExt};
use log::debug;

const WIDTH: u16 = 800;
const HEIGHT: u16 = 800;

struct CollatzShader;

impl ComputeShader for CollatzShader {
    const THE_ONLY_BG_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // camera data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: false,
                    },
                },
            ],
            label: Some("CollatzShader Layout"),
        };

    const THE_ONLY_COMPUTE_MODULE: &'static [u8] =
        include_bytes!("../res/shaders/collatz.comp.spv");
}

struct GeneralDriver {
    pipe: ComputePipeline,
    nums: ManagedBuffer,
    bind_group: wgpu::BindGroup,
    done: bool,
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> Self {
        let nums =
            ManagedBuffer::from_data::<u32>(device, wgpu::BufferUsage::STORAGE, &[1, 4, 3, 295]);

        let pipe = ComputePipeline::new::<CollatzShader>(device);

        let bind_group = BindGroupBuilder::from_layout(pipe.the_only_bg_layout())
            .with_tag("the_only_bind_group")
            .with_buffer(&nums)
            .build(device);

        let done = false;

        Self {
            pipe,
            nums,
            bind_group,
            done,
        }
    }
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        None
    }

    fn update(&mut self, app: &mut Application) -> Vec<wgpu::CommandEncoder> {
        if !self.done {
            let mut encoder =
                app.device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("compute encoder"),
                    });

            debug!("starting compute pass");
            {
                let mut cpass = encoder.begin_compute_pass();
                cpass.set_pipeline(self.pipe.as_ref());
                cpass.set_bind_group(0, &self.bind_group, &[]);
                cpass.dispatch(4, 1, 1);
            }
            debug!("recorded compute pass");
            return vec![encoder];
        }

        vec![]
    }

    fn render(
        &mut self,
        app: &mut Application,
        frame: &wgpu::SwapChainOutput,
    ) -> Vec<wgpu::CommandEncoder> {
        if !self.done {
            debug!("retrieving data");
            let output = executor::block_on(apur::future::post_pending(
                self.nums.read_data::<u32>().boxed(),
                || app.device().poll(wgpu::Maintain::Wait),
            ));
            debug!("data retrived");

            println!("{:?}", output.unwrap().as_slice());

            self.done = true;
        }

        vec![]
    }
}

fn main() {
    env_logger::init();

    let app = executor::block_on(Application::new("solid-shader example", WIDTH, HEIGHT)).unwrap();
    let driver = GeneralDriver::new(app.device());

    app.run(driver);
}
