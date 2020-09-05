use apur::mesh::prefabs::UncoloredTriangle;
use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::{BindGroupLayout, BindGroupLayoutBuilder},
    buffer::ManagedBuffer,
    camera::{Camera, CameraController},
    error as apur_error,
    event_handler::EventHandler,
    pipeline::{RenderPipeline, RenderShader},
};
use futures::{executor, FutureExt};
use std::path::Path;

const WIDTH: u16 = 800;
const HEIGHT: u16 = 800;

struct SolidShader {
    layouts: Vec<BindGroupLayout>,
}

impl SolidShader {
    fn new(device: &wgpu::Device) -> Self {
        let layout = BindGroupLayoutBuilder::new()
            .with_binding(
                wgpu::ShaderStage::VERTEX,
                wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
            )
            .with_binding(
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
            )
            .build(device);

        let layouts = vec![layout];

        Self { layouts }
    }
}

impl RenderShader for SolidShader {
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

    const DEPTH_STENCIL_DESC: Option<wgpu::DepthStencilStateDescriptor> =
        Some(wgpu::DepthStencilStateDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilStateDescriptor {
                front: wgpu::StencilStateFaceDescriptor::IGNORE,
                back: wgpu::StencilStateFaceDescriptor::IGNORE,
                read_mask: !0,
                write_mask: !0,
            },
        });

    fn layouts(&self) -> &[BindGroupLayout] {
        &self.layouts
    }

    fn vertex_module_path(&self) -> &Path {
        "res/shaders/solid/solid.vert.spv".as_ref()
    }

    fn fragment_module_path(&self) -> Option<&Path> {
        Some("res/shaders/solid/solid.frag.spv".as_ref())
    }
}

struct GeneralDriver {
    cam_controller: CameraController,
    pipe: RenderPipeline,
    ds_texture: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    triangle: UncoloredTriangle,
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> apur_error::Result<Self> {
        let mut camera = Camera::new(WIDTH as u32, HEIGHT as u32);
        camera.move_pos(-3.0);
        let cam_controller = CameraController::new(device, camera);

        let color =
            ManagedBuffer::from_data(device, wgpu::BufferUsage::UNIFORM, &[0.7, 0.3, 0.5, 0.0]);

        let shader = SolidShader::new(device);

        let bind_group = shader.layouts()[0]
            .to_bind_group_builder()
            .with_buffer(cam_controller.buffer())?
            .with_buffer(&color)?
            .build(device)?;

        let pipe = RenderPipeline::new(device, &shader)?;
        let ds_texture = device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: WIDTH as u32,
                    height: HEIGHT as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&wgpu::TextureViewDescriptor::default());

        let triangle = UncoloredTriangle::new(device);

        Ok(Self {
            cam_controller,
            pipe,
            ds_texture,
            bind_group,
            triangle,
        })
    }
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        Some(&mut self.cam_controller)
    }

    fn update(&mut self, app: &mut Application) {
        self.cam_controller.update(app.queue()).expect("camera update failed");
    }

    fn render(&mut self, app: &mut Application, frame: &wgpu::SwapChainFrame) {
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
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.ds_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations::default()),
                }),
            });

            rpass.set_pipeline(self.pipe.as_ref());
            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.set_vertex_buffer(0, self.triangle.vertex_buffer().as_ref().slice(..));
            rpass.draw(0..3, 0..1);
        }

        let queue = app.queue();
        queue.submit(vec![encoder.finish()])
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
