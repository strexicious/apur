use apur::model::prefabs::UncoloredTriangle;
use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::BindGroupBuilder,
    buffer::ManagedBuffer,
    camera::{Camera, CameraController},
    event_handler::EventHandler,
    pipeline::{RenderPipeline, RenderShader},
    texture::Texture,
};
use futures::{
    executor,
    future::{self, FutureExt},
    task::{Context, Poll},
};

const WIDTH: u16 = 800;
const HEIGHT: u16 = 800;

struct SolidShader;

impl RenderShader for SolidShader {
    const GLOBAL_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // camera data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: Some("SolidShader GLOBAL_LAYOUT_DESC"),
        };

    const ELEMENT_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // color data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: Some("SolidShader ELEMENT_LAYOUT_DESC"),
        };

    const VERTEX_MODULE: &'static [u8] = include_bytes!("../res/shaders/solid.vert.spv");
    const FRAGMENT_MODULE: &'static [u8] = include_bytes!("../res/shaders/solid.frag.spv");

    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static> = wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: 12,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 0,
                },
            ],
        }],
    };
}

struct GeneralDriver {
    cam_controller: CameraController,
    pipe: RenderPipeline,
    ds_texture: Texture,
    global_bind_g: wgpu::BindGroup,
    element_bind_g: wgpu::BindGroup,
    triangle_1: UncoloredTriangle,
    triangle_2: UncoloredTriangle,
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> Self {
        let mut camera = Camera::new(WIDTH as u32, HEIGHT as u32);
        camera.move_pos(-3.0);
        let cam_controller = CameraController::new(device, camera);

        let color =
            ManagedBuffer::from_data(device, wgpu::BufferUsage::UNIFORM, &[0.7, 0.3, 0.5, 0.0]);

        let pipe = RenderPipeline::new::<SolidShader>(device);
        let ds_texture = Texture::new_depth(device, WIDTH as u32, HEIGHT as u32);

        let global_bind_g = BindGroupBuilder::from_layout(pipe.global_layout())
            .with_tag("global_bind_group")
            .with_buffer(cam_controller.buffer())
            .build(device);
        let element_bind_g = BindGroupBuilder::from_layout(pipe.element_layout())
            .with_tag("element_bind_group")
            .with_buffer(&color)
            .build(device);

        let triangle_1 = UncoloredTriangle::new(device);
        let triangle_2 = UncoloredTriangle::new(device);

        Self {
            cam_controller,
            pipe,
            ds_texture,
            global_bind_g,
            element_bind_g,
            triangle_1,
            triangle_2,
        }
    }

    async fn buffer_updates(&mut self) {
        self.cam_controller.update().await;
    }
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        Some(&mut self.cam_controller)
    }

    fn update(&mut self, app: &mut Application) -> Vec<wgpu::CommandEncoder> {
        executor::block_on(apur::future::post_pending(
            self.buffer_updates().boxed(),
            || app.device().poll(wgpu::Maintain::Poll),
        ));

        vec![]
    }

    fn render(
        &mut self,
        app: &mut Application,
        frame: &wgpu::SwapChainOutput,
    ) -> Vec<wgpu::CommandEncoder> {
        let mut encoder = app
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        const CLEAR_COLOR: wgpu::Color = wgpu::Color {
            r: 0.2,
            g: 0.5,
            b: 0.7,
            a: 1.0,
        };

        // render pass goes in a block because it needs to be
        // dropped before we can reuse encoder as it was borrowed
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: CLEAR_COLOR,
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: self.ds_texture.view(),
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Load,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                }),
            });

            rpass.set_pipeline(self.pipe.as_ref());
            rpass.set_bind_group(0, &self.global_bind_g, &[]);
            rpass.set_bind_group(1, &self.element_bind_g, &[]);

            rpass.set_vertex_buffer(
                0,
                self.triangle_1.vertex_buffer().as_ref(),
                0,
                self.triangle_1.vertex_buffer().size_bytes() as u64,
            );
            rpass.draw(0..3, 0..1);
        }

        vec![encoder]
    }
}

fn main() {
    env_logger::init();

    let app = executor::block_on(Application::new("solid-shader example", WIDTH, HEIGHT)).unwrap();
    let driver = GeneralDriver::new(app.device());

    app.run(driver);
}
