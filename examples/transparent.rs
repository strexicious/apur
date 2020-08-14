use std::path::Path;
use apur::model::{
    mesh::{self, Mesh},
    prefabs::UncoloredCube,
};
use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::{BindGroupLayout, BindGroupLayoutBuilder},
    buffer::ManagedBuffer,
    camera::{Camera, CameraController},
    error as apur_error,
    event_handler::EventHandler,
    pipeline::{RenderPipeline, RenderShader},
    texture::{DepthTexture, Texture},
};
use futures::{executor, FutureExt};

const WIDTH: u16 = 800;
const HEIGHT: u16 = 800;

struct TransparentShader {
    layouts: Vec<BindGroupLayout>,
}

impl TransparentShader {
    fn new(device: &wgpu::Device) -> Self {
        let camera_layout = BindGroupLayoutBuilder::new()
            .with_binding(
                wgpu::ShaderStage::VERTEX,
                wgpu::BindingType::UniformBuffer { dynamic: false },
            )
            .build(device);

        let objects_layout = BindGroupLayoutBuilder::new()
            .with_binding(
                wgpu::ShaderStage::VERTEX,
                wgpu::BindingType::UniformBuffer { dynamic: false },
            )
            .with_binding(
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::UniformBuffer { dynamic: false },
            )
            .build(device);

        let layouts = vec![camera_layout, objects_layout];

        Self { layouts }
    }
}

impl RenderShader for TransparentShader {
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

    const DEPTH_STENCIL_DESC: Option<wgpu::DepthStencilStateDescriptor> =
        Some(wgpu::DepthStencilStateDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: !0,
            stencil_write_mask: !0,
        });

    const COLOR_STATE_DESCS: &'static [wgpu::ColorStateDescriptor] =
        &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

    fn layouts(&self) -> &[BindGroupLayout] {
        &self.layouts
    }

    fn vertex_module_path(&self) -> &Path {
        "res/shaders/transparent.vert.spv".as_ref()
    }

    fn fragment_module_path(&self) -> &Path {
        "res/shaders/transparent.frag.spv".as_ref()
    }
}

struct GeneralDriver {
    cam_controller: CameraController,
    pipe: RenderPipeline,
    ds_texture: DepthTexture,
    cam_bind_group: wgpu::BindGroup,
    cup: (Mesh, wgpu::BindGroup),
    cube1: (UncoloredCube, wgpu::BindGroup),
    cube2: (UncoloredCube, wgpu::BindGroup),
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> apur_error::Result<Self> {
        let mut camera = Camera::new(WIDTH as u32, HEIGHT as u32);
        camera.move_pos(-10.0);
        let cam_controller = CameraController::new(device, camera);

        let shader = TransparentShader::new(device);

        let cam_bind_group = shader.layouts()[0]
            .to_bind_group_builder()
            .with_buffer(cam_controller.buffer())?
            .build(device)?;

        let mut cup_model = mesh::load_model(device, "fab_cup");

        let cup_bg = shader.layouts()[1]
            .to_bind_group_builder()
            .with_buffer(&ManagedBuffer::from_data::<f32>(
                device,
                wgpu::BufferUsage::UNIFORM,
                &[1.0],
            ))?
            .with_buffer(&ManagedBuffer::from_data::<f32>(
                device,
                wgpu::BufferUsage::UNIFORM,
                &[1.0, 1.0, 0.0, 1.0],
            ))?
            .build(device)?;

        let cube1_bg = shader.layouts()[1]
            .to_bind_group_builder()
            .with_buffer(&ManagedBuffer::from_data::<f32>(
                device,
                wgpu::BufferUsage::UNIFORM,
                &[3.0],
            ))?
            .with_buffer(&ManagedBuffer::from_data::<f32>(
                device,
                wgpu::BufferUsage::UNIFORM,
                &[1.0, 0.0, 1.0, 0.3],
            ))?
            .build(device)?;

        let cube2_bg = shader.layouts()[1]
            .to_bind_group_builder()
            .with_buffer(&ManagedBuffer::from_data::<f32>(
                device,
                wgpu::BufferUsage::UNIFORM,
                &[5.0],
            ))?
            .with_buffer(&ManagedBuffer::from_data::<f32>(
                device,
                wgpu::BufferUsage::UNIFORM,
                &[0.0, 1.0, 1.0, 0.1],
            ))?
            .build(device)?;

        let cup = (cup_model.pop().unwrap(), cup_bg);
        let cube1 = (UncoloredCube::new(device), cube1_bg);
        let cube2 = (UncoloredCube::new(device), cube2_bg);

        let pipe = RenderPipeline::new(device, &shader).unwrap();
        let ds_texture = DepthTexture::new(device, WIDTH as u32, HEIGHT as u32);

        Ok(Self {
            cam_controller,
            pipe,
            ds_texture,
            cam_bind_group,
            cup,
            cube1,
            cube2,
        })
    }
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        Some(&mut self.cam_controller)
    }

    fn update(&mut self, app: &mut Application) -> Vec<wgpu::CommandBuffer> {
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
    ) -> Vec<wgpu::CommandBuffer> {
        let mut encoder = app
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        const CLEAR_COLOR: wgpu::Color = wgpu::Color {
            r: 0.02,
            g: 0.05,
            b: 0.07,
            a: 1.00,
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
            rpass.set_bind_group(0, &self.cam_bind_group, &[]);

            rpass.set_bind_group(1, &self.cup.1, &[]);
            rpass.set_vertex_buffer(
                0,
                self.cup.0.positions_buffer().as_ref(),
                0,
                self.cup.0.positions_buffer().size_bytes() as u64,
            );
            rpass.set_index_buffer(
                self.cup.0.indices_buffer().as_ref(),
                0,
                self.cup.0.indices_buffer().size_bytes() as u64,
            );
            rpass.draw_indexed(
                0..self.cup.0.indices_buffer().size_bytes() as u32 / 4,
                0,
                0..1,
            );

            let cube1_buf = self.cube1.0.vertex_buffer();
            rpass.set_bind_group(1, &self.cube1.1, &[]);
            rpass.set_vertex_buffer(0, cube1_buf.as_ref(), 0, cube1_buf.size_bytes() as u64);
            rpass.draw(0..cube1_buf.size_bytes() as u32 / 4 / 3, 0..1);

            let cube2_buf = self.cube2.0.vertex_buffer();
            rpass.set_bind_group(1, &self.cube2.1, &[]);
            rpass.set_vertex_buffer(0, cube2_buf.as_ref(), 0, cube2_buf.size_bytes() as u64);
            rpass.draw(0..cube2_buf.size_bytes() as u32 / 4 / 3, 0..1);
        }

        vec![encoder.finish()]
    }
}

fn main() {
    env_logger::init();

    let app = executor::block_on(Application::new(
        "transparent-shader example",
        WIDTH,
        HEIGHT,
    ))
    .unwrap();
    let driver = GeneralDriver::new(app.device());

    app.run(driver.unwrap());
}
