use std::io::Read;
use std::fs::File;
use winit::window::Window;
use glam::{Mat4};

mod camera;

use camera::{Camera, Frustum};

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const fn get_attribs_layout() -> [wgpu::VertexAttributeDescriptor; 2] {
        [
            wgpu::VertexAttributeDescriptor {
                offset: 0,
                format: wgpu::VertexFormat::Float3,
                shader_location: 0,
            },
            wgpu::VertexAttributeDescriptor {
                offset: 12,
                format: wgpu::VertexFormat::Float2,
                shader_location: 1,
            },
        ]
    }
}

struct Mesh {
    vertices: Vec<Vertex>,
    vbuffer: wgpu::Buffer,
    indices: Vec<u32>,
    ibuffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

impl Mesh {
    fn new(
        device: &wgpu::Device,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        texture: wgpu::Texture,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        uniforms_buffer: &wgpu::Buffer,
    ) -> Self {
        let vbuffer = device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);
        let ibuffer = device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices);
        let texture_view = texture.create_default_view();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: uniforms_buffer,
                        range: 0 .. 3 * 64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        Self { vertices, vbuffer, indices, ibuffer, texture, texture_view, bind_group }
    }
    
    fn draw(&self, pass: &mut wgpu::RenderPass) {
        pass.set_vertex_buffers(0, &[(&self.vbuffer, 0)]);
        pass.set_index_buffer(&self.ibuffer, 0);
        pass.draw_indexed(0 .. self.indices.len() as u32, 0, 0 .. 1);
    }
}

struct ModelManager;

impl ModelManager {
    // TODO: maybe nicer this API and this should be an instance function
    fn load_models(device: &wgpu::Device, queue: &mut wgpu::Queue, obj_filename: &str, bind_group_layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, uniforms_buffer: &wgpu::Buffer) -> Vec<Mesh> {
        let (models, mats) = tobj::load_obj(format!("res/models/{}.obj", obj_filename).as_ref()).expect("Failed to load the model");
        
        let mut cmd_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        
        let meshes = models.into_iter().map(|m| {
            assert_eq!(m.mesh.positions.len() / 3, m.mesh.texcoords.len() / 2, "positions and texcoords length not same");

            let vertices = m.mesh.positions
                .chunks(3)
                .zip(m.mesh.texcoords.chunks(2))
                .map(|(vs, ts)| Vertex {
                    pos: [vs[0], vs[1], vs[2]],
                    tex_coords: [ts[0], ts[1]],
                })
                .collect::<Vec<Vertex>>();
            let mat_idx = m.mesh.material_id.expect("no material associated");
            assert!(!mats[mat_idx].diffuse_texture.is_empty(), "diffuse texture path empty");
            let texture = Self::load_texture(device, &mut cmd_encoder, &mats[mat_idx].diffuse_texture);

            Mesh::new(&device, vertices, m.mesh.indices, texture, bind_group_layout, sampler, uniforms_buffer)
        }).collect();

        queue.submit(&[cmd_encoder.finish()]);

        meshes
    }

    fn load_texture(device: &wgpu::Device, cmd_encoder: &mut wgpu::CommandEncoder, name: &str) -> wgpu::Texture {
        // assumed name includes the "texture/" in path name
        println!("[Info] Loading texture: {}", name);
        let mut image_file = File::open(format!("res/models/{}", name)).expect("Failed to open texture image");
        let mut image_contents = vec![];
        let _ = image_file.read_to_end(&mut image_contents);
        
        let texture_image = image::load_from_memory(&image_contents)
            .expect("failed to load a texture image");
        let texture_image = texture_image.into_rgba();
        
        let texture_extent = wgpu::Extent3d {
            width: texture_image.width(),
            height: texture_image.height(),
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let image_width = texture_image.width();
        let image_height = texture_image.height();
        println!("[Info] Texture has size: {} {}", image_width, image_height);
        let image_data = texture_image.into_vec();
        let image_buf = device
            .create_buffer_mapped(image_data.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&image_data);

        cmd_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &image_buf,
                offset: 0,
                row_pitch: 4 * image_width,
                image_height,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d { x: 0f32, y: 0f32, z: 0f32 },
            },
            texture_extent
        );

        texture
    }
}

struct Renderer {
    meshes: Vec<Mesh>,
    uniforms_buffer: wgpu::Buffer,
    binds_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}

impl Renderer {
    fn new(device: &wgpu::Device, view_trans: Mat4, proj_trans: Mat4) -> Self {
        let vs_source = include_bytes!("../res/shaders/shader.vert.spv");
        let vs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&vs_source[..])).expect("failed to read vertex shader spir-v"));

        let fs_source = include_bytes!("../res/shaders/shader.frag.spv");
        let fs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&fs_source[..])).expect("failed to read fragment shader spir-v"));
            
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                }
            ]
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil_front: wgpu::StencilStateFaceDescriptor::default(),
                stencil_back: wgpu::StencilStateFaceDescriptor::default(),
                stencil_read_mask: !0,
                stencil_write_mask: !0,
            }),
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &Vertex::get_attribs_layout(),
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let uniforms_buffer = device
            .create_buffer_mapped(3, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[Mat4::identity(), view_trans, proj_trans]);
            
        Self {
            meshes: vec![],
            binds_layout: bind_group_layout,
            pipeline,
            sampler,
            uniforms_buffer,
        }
    }

    fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    fn render(&self, frame: &wgpu::SwapChainOutput, cmd_encoder: &mut wgpu::CommandEncoder, depth_texture_view: &wgpu::TextureView) {
        
        const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.4, g: 0.1, b: 0.1, a: 1.0 };
        
        let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: CLEAR_COLOR,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth_texture_view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });
        rpass.set_pipeline(&self.pipeline);

        for mesh in &self.meshes {
            rpass.set_bind_group(0, &mesh.bind_group, &[]);
            mesh.draw(&mut rpass);
        }
    }
}

pub struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    renderer: Renderer,
    camera: Camera,
    frustum: Frustum,
}

impl Engine {
    pub fn new(window: &Window) -> Self {
        let WINDOW_WIDTH = window.inner_size().width;
        let WINDOW_HEIGHT = window.inner_size().height;
        
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,
        }).expect("Couldn't get hardware adapter");
        let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions::default(),
            limits: wgpu::Limits::default(),
        });
        
        let surface = wgpu::Surface::create(window);
        let swapchain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            present_mode: wgpu::PresentMode::Vsync,
        });

        let mut camera = Camera::default();
        camera.move_pos(-5.0);
        let frustum = Frustum::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width: WINDOW_WIDTH, height: WINDOW_HEIGHT, depth: 1, },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let depth_texture_view = depth_texture.create_default_view();

        let mut renderer = Renderer::new(&device, camera.view(), frustum.projection());
        for mesh in ModelManager::load_models(&device, &mut queue, "Planks of wood", &renderer.binds_layout, &renderer.sampler, &renderer.uniforms_buffer) {
            renderer.add_mesh(mesh);
        }

        Self { device, queue, swapchain, depth_texture, depth_texture_view, renderer, camera, frustum }
    }

    pub fn render(&mut self) {
        let frame = self.swapchain.get_next_texture();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.renderer.render(&frame, &mut encoder, &self.depth_texture_view);
        self.queue.submit(&[encoder.finish()]);
    }
}
