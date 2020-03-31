use glam::{Mat4};

use std::io::Read;
use std::fs::File;

use super::super::model::Vertex;

pub struct SkyBoxRenderer {
    background_plane: wgpu::Buffer,
    transforms_buffer: wgpu::Buffer,
    cubemap_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl SkyBoxRenderer {
    pub fn new(
        device: &wgpu::Device,
        cubemap_name: &str,
        queue: &mut wgpu::Queue,
        view_trans: Mat4,
        proj_trans: Mat4,
    ) -> Self {
        let vs_source = include_bytes!("../../../res/shaders/skybox.vert.spv");
        let vs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&vs_source[..])).expect("failed to read vertex shader spir-v"));

        let fs_source = include_bytes!("../../../res/shaders/skybox.frag.spv");
        let fs_module = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&fs_source[..])).expect("failed to read fragment shader spir-v"));

        let cubemap_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::CubeArray,
                    },
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ]
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&cubemap_bind_group_layout],
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
                front_face: wgpu::FrontFace::Ccw,
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
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<f32>() as u64 * 3,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 0,
                    },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        
        let mut cmd_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        println!("[Info] Loading cubemap for skybox: {}", cubemap_name);
        
        const MAP_SUFFIXES: [&str; 6] = ["_px.jpg", "_nx.jpg", "_py.jpg", "_ny.jpg", "_pz.jpg", "_nz.jpg"];
        
        let mut cubemap_width = 0;
        let mut cubemap_height = 0;
        let cubemap_maps = MAP_SUFFIXES.iter()
            .map(|suf| format!("{}{}", cubemap_name, suf))
            .map(|side_name| {
                let mut image_file = File::open(format!("res/models/textures/{}", side_name)).expect("Failed to open cubemap image");
                let mut image_contents = vec![];
                let _ = image_file.read_to_end(&mut image_contents);
                
                let cubemap_image = image::load_from_memory(&image_contents)
                    .expect("failed to load a cubemap image")
                    .into_rgba();
                
                cubemap_width = cubemap_image.width();
                cubemap_height = cubemap_image.height();

                cubemap_image
            })
            .collect::<Vec<_>>();

        let texture_extent = wgpu::Extent3d {
            width: cubemap_width,
            height: cubemap_height,
            depth: 1,
        };

        let cubemap_view = {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size: texture_extent,
                array_layer_count: 6,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            });
            
            let cubemap_data = cubemap_maps.into_iter().map(|m| m.into_vec()).flatten().collect::<Vec<_>>();
            let cubemap_buf = device
                .create_buffer_mapped(cubemap_data.len(), wgpu::BufferUsage::COPY_SRC)
                .fill_from_slice(&cubemap_data);
        
            for i in 0..=5 {
                cmd_encoder.copy_buffer_to_texture(
                    wgpu::BufferCopyView {
                        buffer: &cubemap_buf,
                        offset: (i as u64) * 4 * (cubemap_width as u64) * (cubemap_height as u64),
                        row_pitch: 4 * cubemap_width,
                        image_height: cubemap_height,
                    },
                    wgpu::TextureCopyView {
                        texture: &texture,
                        mip_level: 0,
                        array_layer: i as u32,
                        origin: wgpu::Origin3d { x: 0f32, y: 0f32, z: 0f32 },
                    },
                    texture_extent
                );
            }

            texture.create_default_view()
        };
        
        queue.submit(&[cmd_encoder.finish()]);

        let background_plane_verts: Vec<f32> = vec![
            -1.0,  1.0, -1.0,
            -1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,
        
            -1.0, -1.0,  1.0,
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
            -1.0, -1.0,  1.0,
        
             1.0, -1.0, -1.0,
             1.0, -1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0, -1.0,
             1.0, -1.0, -1.0,
        
            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,
        
            -1.0,  1.0, -1.0,
             1.0,  1.0, -1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0,
        
            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0,  1.0,
        ];
        
        let background_plane = device
            .create_buffer_mapped(background_plane_verts.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&background_plane_verts);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let transforms_buffer = device
            .create_buffer_mapped(2, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_WRITE)
            .fill_from_slice(&[view_trans, proj_trans]);

        let cubemap_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &cubemap_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cubemap_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &transforms_buffer,
                        range: 0 .. 2 * 64,
                    },
                },
            ]
        });

        Self { background_plane, cubemap_bind_group, pipeline, transforms_buffer }
    }

    pub fn render(&self, frame: &wgpu::SwapChainOutput, cmd_encoder: &mut wgpu::CommandEncoder) {
        let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.cubemap_bind_group, &[]);
        rpass.set_vertex_buffers(0, &[(&self.background_plane, 0)]);

        rpass.draw(0..36, 0..1);
    }

    pub fn get_transforms_buffer(&self) -> &wgpu::Buffer {
        &self.transforms_buffer
    }
}
