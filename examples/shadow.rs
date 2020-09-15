use apur::renderer::{
    application::{Application, ApplicationDriver},
    bind_group::{BindGroupLayout, BindGroupLayoutBuilder},
    buffer::ManagedBuffer,
    camera::{Camera, CameraController},
    error as apur_error,
    event_handler::EventHandler,
    pipeline::{RenderPipeline, RenderShader},
};
use apur::{
    light::DirectionalLight,
    mesh::{Mesh, Model},
};
use futures::{executor, FutureExt};
use notify::{RecursiveMode, Watcher};
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

const WIDTH: u16 = 800;
const HEIGHT: u16 = 800;
struct LightShader {
    layouts: Vec<BindGroupLayout>,
}

impl LightShader {
    fn new(device: &wgpu::Device) -> Self {
        let layout = BindGroupLayoutBuilder::new()
            .with_binding(
                wgpu::ShaderStage::VERTEX,
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

impl RenderShader for LightShader {
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

    const COLOR_STATE_DESCS: &'static [wgpu::ColorStateDescriptor] = &[];

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
        "./res/shaders/lightview/lightview.vert.spv".as_ref()
    }

    fn fragment_module_path(&self) -> Option<&Path> {
        None
    }
}

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
            .with_binding(
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::Sampler { comparison: true },
            )
            .with_binding(
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::SampledTexture {
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Float,
                    multisampled: false,
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

    const COLOR_STATE_DESCS: &'static [wgpu::ColorStateDescriptor] =
        &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::Zero,
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
        "res/shaders/shadow/shadow.vert.spv".as_ref()
    }

    fn fragment_module_path(&self) -> Option<&Path> {
        Some("res/shaders/shadow/shadow.frag.spv".as_ref())
    }
}

struct GeneralDriver {
    cam_controller: CameraController,
    meshes: Model,
    light_s: LightShader,
    light_p: RenderPipeline,
    final_s: SolidShader,
    final_p: RenderPipeline,
    ds_texture: wgpu::TextureView,
    shadow_map: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    final_bind_group: wgpu::BindGroup,
    recreate_pipeline: Arc<AtomicBool>,
    shader_watcher: notify::RecommendedWatcher,
}

impl GeneralDriver {
    fn new(device: &wgpu::Device) -> apur_error::Result<Self> {
        // setup scene
        let mut camera = Camera::new(WIDTH as u32, HEIGHT as u32);
        camera.move_pos(-3.0);
        let cam_controller = CameraController::new(device, camera);
        let meshes = Model::load(device, "wide_monkey");
        let light = DirectionalLight::new(device, (-1.0, -1.0, 0.0), meshes.bounding_box());

        // setup light
        let light_s = LightShader::new(device);
        let light_p = RenderPipeline::new(device, &light_s)?;

        // setup final shader
        let final_s = SolidShader::new(device);
        let final_p = RenderPipeline::new(device, &final_s)?;

        // setup gpu resources
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
        let shadow_map = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("shadow_map"),
                size: wgpu::Extent3d {
                    width: WIDTH as u32,
                    height: HEIGHT as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            label: None,
            compare: Some(wgpu::CompareFunction::Less),
            anisotropy_clamp: None,
        });

        let bind_group = light_s.layouts()[0]
            .to_bind_group_builder()
            .with_buffer(light.uniform_buffer())?
            .build(device)?;

        let final_bind_group = final_s.layouts()[0]
            .to_bind_group_builder()
            .with_buffer(cam_controller.buffer())?
            .with_buffer(light.uniform_buffer())?
            .with_sampler(&sampler)?
            .with_texture(&shadow_map)?
            .build(device)?;

        // dynamic shader logic
        let recreate_pipeline = Arc::new(AtomicBool::new(false));
        let rec_pipe_clone = recreate_pipeline.clone();

        let mut shader_watcher = notify::immediate_watcher(move |_e| {
            rec_pipe_clone.store(true, Ordering::Relaxed);
        })
        .unwrap();

        shader_watcher
            .watch(
                "res/shaders/lightview/lightview.vert.spv",
                RecursiveMode::NonRecursive,
            )
            .unwrap();

        Ok(Self {
            cam_controller,
            meshes,
            light_s,
            light_p,
            final_s,
            final_p,
            ds_texture,
            shadow_map,
            bind_group,
            final_bind_group,
            recreate_pipeline,
            shader_watcher,
        })
    }

    fn light_pass(
        &mut self,
        app: &mut Application,
        frame: &wgpu::SwapChainFrame,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.shadow_map,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations::default()),
            }),
        });

        rpass.set_pipeline(self.light_p.as_ref());
        rpass.set_bind_group(0, &self.bind_group, &[]);

        for mesh in self.meshes.meshes() {
            let p = mesh.positions_buffer();
            let i = mesh.indices_buffer();

            rpass.set_vertex_buffer(0, p.as_ref().slice(..));
            rpass.set_index_buffer(i.as_ref().slice(..));
            rpass.draw_indexed(0..i.size_bytes() as u32 / 4, 0, 0..1);
        }
    }

    fn final_pass(
        &mut self,
        app: &mut Application,
        frame: &wgpu::SwapChainFrame,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        const CLEAR_COLOR: wgpu::Color = wgpu::Color {
            r: 0.2,
            g: 0.5,
            b: 0.7,
            a: 1.0,
        };

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

        rpass.set_pipeline(self.final_p.as_ref());
        rpass.set_bind_group(0, &self.final_bind_group, &[]);

        for mesh in self.meshes.meshes() {
            let p = mesh.positions_buffer();
            let n = mesh.normals_buffer().unwrap();
            let i = mesh.indices_buffer();

            rpass.set_vertex_buffer(0, p.as_ref().slice(..));
            rpass.set_vertex_buffer(1, n.as_ref().slice(..));
            rpass.set_index_buffer(i.as_ref().slice(..));
            rpass.draw_indexed(0..i.size_bytes() as u32 / 4, 0, 0..1);
        }
    }
}

impl ApplicationDriver for GeneralDriver {
    fn current_event_handler(&mut self, _app: &mut Application) -> Option<&mut dyn EventHandler> {
        Some(&mut self.cam_controller)
    }

    fn update(&mut self, app: &mut Application) {
        if self.recreate_pipeline.load(Ordering::Relaxed) {
            self.light_p = RenderPipeline::new(app.device(), &self.light_s).unwrap();
            self.final_p = RenderPipeline::new(app.device(), &self.final_s).unwrap();
            self.recreate_pipeline.store(false, Ordering::Relaxed);
        }
        self.cam_controller.update(app.queue()).unwrap();
    }

    fn render(&mut self, app: &mut Application, frame: &wgpu::SwapChainFrame) {
        let mut encoder = app
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        self.light_pass(app, frame, &mut encoder);
        self.final_pass(app, frame, &mut encoder);

        let queue = app.queue();
        queue.submit(vec![encoder.finish()])
    }
}

// It works, but typical problems with shadow maps appear.
// Most notably self-shadowing is present. This requires changes
// in the rasterization state of the pipeline, which is currently hardcoded
// and is same for all RenderPipeline objects. If you wish to execute this example,
// you may try out changing the sloped depth bias to 1.0.
fn main() {
    env_logger::init();

    let app = executor::block_on(Application::new("solid-shader example", WIDTH, HEIGHT)).unwrap();

    match GeneralDriver::new(app.device()) {
        Ok(driver) => app.run(driver),
        Err(e) => eprintln!("startup error: {:?}", e),
    }
}
