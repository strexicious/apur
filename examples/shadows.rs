use apur::model::{lights::DirectionalLight, mesh::{self, Mesh}, prefabs::UncoloredTriangle};
use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::{BindGroupLayout, BindGroupLayoutBuilder},
    buffer::ManagedBuffer,
    camera::{Camera, CameraController},
    error as apur_error,
    event_handler::EventHandler,
    pipeline::{RenderPipeline, RenderShader},
    texture::Texture,
};
use futures::{executor, FutureExt};

const WIDTH: u16 = 800;
const HEIGHT: u16 = 800;

struct ShadowShader {
    layouts: Vec<BindGroupLayout>,
}

impl ShadowShader {
    fn new(device: &wgpu::Device) -> Self {
        let layout = BindGroupLayoutBuilder::new()
            .with_binding(
                wgpu::ShaderStage::VERTEX,
                wgpu::BindingType::UniformBuffer { dynamic: false },
            )
            .with_binding(
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::UniformBuffer { dynamic: false },
            )
            .build(device);

        let layouts = vec![layout];

        Self { layouts }
    }
}

impl RenderShader for ShadowShader {
    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static> = wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[
            wgpu::VertexBufferDescriptor {
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
            },
            wgpu::VertexBufferDescriptor {
                stride: 12,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    // normal
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 1,
                    },
                ],
            },
        ],
    };

    fn layouts(&self) -> &[BindGroupLayout] {
        &self.layouts
    }

    fn vertex_module(&self) -> &[u8] {
        include_bytes!("../res/shaders/shadow.vert.spv")
    }

    fn fragment_module(&self) -> &[u8] {
        include_bytes!("../res/shaders/shadow.frag.spv")
    }
}

struct GeneralDriver {
    cam_controller: CameraController,
    pipe: RenderPipeline,
    ds_texture: Texture,
    bind_group: wgpu::BindGroup,
    meshes: Vec<Mesh>,
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> apur_error::Result<Self> {
        let mut camera = Camera::new(WIDTH as u32, HEIGHT as u32);
        camera.move_pos(-3.0);
        let cam_controller = CameraController::new(device, camera);

        let shader = ShadowShader::new(device);

        let light = DirectionalLight::new(device, (-1.0, -1.0, 0.0));
        let bind_group = shader.layouts()[0]
            .to_bind_group_builder()
            .with_buffer(cam_controller.buffer())?
            .with_buffer(light.uniform_buffer())?
            .build(device)?;

        let pipe = RenderPipeline::new(device, shader);
        let ds_texture = Texture::new_depth(device, WIDTH as u32, HEIGHT as u32);

        let meshes = mesh::load_model(device, "wide_monkey");

        Ok(Self {
            cam_controller,
            pipe,
            ds_texture,
            bind_group,
            meshes,
        })
    }
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        Some(&mut self.cam_controller)
    }

    fn update(&mut self, app: &mut Application) -> Vec<wgpu::CommandEncoder> {
        executor::block_on(apur::future::post_pending(
            self.cam_controller.update().boxed(),
            // difference between Poll and Wait:
            // - Poll: resolve the futures for mappings that
            //   are already done, and quit
            // - Wait: wait for all mappings to be done
            //   in order to resolve all pending futures
            || app.device().poll(wgpu::Maintain::Wait),
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
            rpass.set_bind_group(0, &self.bind_group, &[]);

            for mesh in &self.meshes {
                let p = mesh.positions_buffer();
                let n = mesh.normals_buffer().expect("normals were empty");
                let i = mesh.indices_buffer();

                rpass.set_vertex_buffer(0, p.as_ref(), 0, p.size_bytes() as u64);
                rpass.set_vertex_buffer(1, n.as_ref(), 0, n.size_bytes() as u64);
                rpass.set_index_buffer(i.as_ref(), 0, i.size_bytes() as u64);
                rpass.draw_indexed(0..i.size_bytes() as u32 / 4, 0, 0..1);
            }
        }

        vec![encoder]
    }
}

fn main() {
    env_logger::init();

    let app = executor::block_on(Application::new("solid-shader example", WIDTH, HEIGHT)).unwrap();
    let driver = GeneralDriver::new(app.device());

    app.run(driver.unwrap());
}
