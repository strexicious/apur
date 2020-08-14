use std::path::Path;
use apur::model::{
    lights::DirectionalLight,
    mesh::{self, Mesh},
    prefabs::UncoloredTriangle,
};
use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::{BindGroupLayout, BindGroupLayoutBuilder},
    buffer::ManagedBuffer,
    camera::{Camera, CameraController},
    error as apur_error,
    event_handler::EventHandler,
    pipeline::{RenderPipeline, RenderShader},
    texture::{DepthTexture, FragmentOutputTexture, Texture},
};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use futures::{executor, FutureExt};
use notify::{Watcher, RecursiveMode};

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
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
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

    const COLOR_STATE_DESCS: &'static [wgpu::ColorStateDescriptor] = &[
        wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        },
        wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba32Float,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        },
    ];

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

    fn layouts(&self) -> &[BindGroupLayout] {
        &self.layouts
    }

    fn vertex_module_path(&self) -> &Path {
        "res/shaders/shadow.vert.spv".as_ref()
    }

    fn fragment_module_path(&self) -> &Path {
        "res/shaders/shadow.frag.spv".as_ref()
    }
}

struct GeneralDriver {
    cam_controller: CameraController,
    pipe: RenderPipeline,
    ds_texture: DepthTexture,
    sample_points_texture: FragmentOutputTexture,
    bind_group: wgpu::BindGroup,
    meshes: Vec<Mesh>,
    sshader: ShadowShader,
    recreate_pipeline: Arc<AtomicBool>,
    shader_watcher: notify::RecommendedWatcher,
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> apur_error::Result<Self> {
        let mut camera = Camera::new(WIDTH as u32, HEIGHT as u32);
        camera.move_pos(-3.0);
        let cam_controller = CameraController::new(device, camera);

        let light = DirectionalLight::new(device, (-1.0, -1.0, 0.0));

        let sshader = ShadowShader::new(device);
        let bind_group = sshader.layouts()[0]
            .to_bind_group_builder()
            .with_buffer(cam_controller.buffer())?
            .with_buffer(light.uniform_buffer())?
            .build(device)?;

        let pipe = RenderPipeline::new(device, &sshader).unwrap();
        let ds_texture = DepthTexture::new(device, WIDTH as u32, HEIGHT as u32);
        let sample_points_texture = FragmentOutputTexture::new(device, WIDTH as u32, HEIGHT as u32);

        let meshes = mesh::load_model(device, "wide_monkey");

        let recreate_pipeline = Arc::new(AtomicBool::new(false));
        let rec_pipe_clone = recreate_pipeline.clone();

        let mut shader_watcher = notify::immediate_watcher(move |_e| {
            rec_pipe_clone.store(true, Ordering::Relaxed);
        }).unwrap();
        shader_watcher.watch("res/shaders/shadow.vert.spv", RecursiveMode::NonRecursive).unwrap();
        shader_watcher.watch("res/shaders/shadow.frag.spv", RecursiveMode::NonRecursive).unwrap();

        Ok(Self {
            cam_controller,
            pipe,
            ds_texture,
            sample_points_texture,
            bind_group,
            meshes,
            sshader,
            recreate_pipeline,
            shader_watcher,
        })
    }

    fn first_shadowless_pass(
        &mut self,
        app: &mut Application,
        frame: &wgpu::SwapChainOutput,
        encoder: &mut wgpu::CommandEncoder,
    ) { 
        const CLEAR_COLOR: wgpu::Color = wgpu::Color {
            r: 0.2,
            g: 0.5,
            b: 0.7,
            a: 1.0,
        };

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: CLEAR_COLOR,
                },
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.sample_points_texture.view(),
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                },
            ],
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
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        Some(&mut self.cam_controller)
    }

    fn update(&mut self, app: &mut Application) -> Vec<wgpu::CommandBuffer> {
        if self.recreate_pipeline.load(Ordering::Relaxed) {
            self.pipe = RenderPipeline::new(app.device(), &self.sshader).unwrap();
            self.recreate_pipeline.store(false, Ordering::Relaxed);
        }
        
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
        
        self.first_shadowless_pass(app, frame, &mut encoder);

        vec![encoder.finish()]
    }
}

fn main() {
    env_logger::init();

    let app = executor::block_on(Application::new("solid-shader example", WIDTH, HEIGHT)).unwrap();
    let driver = GeneralDriver::new(app.device());

    app.run(driver.unwrap());
}
