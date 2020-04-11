use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::collections::HashMap;

// Maybe we can define a trait that will represent the dependece of
// a material on a vertex structure but at the same time will
// will result in a product that would be the pipeline, something like
// trait Material {
//     // Vertex would have declare an associated method
//     // that returns the vertex state for our pipeline
//     type VertexType: Vertex;

//     fn get_pipeline(&self) -> &wgpu::RenderPipeline;
// }
// we will also need to figure out a good way to store references to our models
// so we can efficiently manually bind the pipeline only once for each material
// type and then render the referenced models in the same subpass, maybe
// when creating the material, also pass in the mess to the manager which does the desired

struct MaterialProps {
    metalness: f32,
    roughness: f32,
}

struct MaterialMaps {
    fixed_albedo: [f32; 3],
    specular: Option<Rc<wgpu::TextureView>>,
    diffuse: Option<Rc<wgpu::TextureView>>,
}

struct Material {
    mat_props: MaterialProps,
    mat_maps: MaterialMaps,
}

pub struct MaterialManager {
    loaded_maps: HashMap<String, Rc<wgpu::TextureView>>,
    loaded_materials: HashMap<String, Material>,
    // fixed_albedo pipeline
    fa_pipe: wgpu::RenderPipeline,
    // specular pipeline
    sp_pipe: wgpu::RenderPipeline,
    // diffuse pipeline
    df_pipe: wgpu::RenderPipeline,
    // combined pipeline
    comb_pipe: wgpu::RenderPipeline,
}

impl MaterialManager {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            loaded_maps: HashMap::new(),
            loaded_materials: HashMap::new(),
            fa_pipe: fa_pipeline(device),
            sp_pipe: sp_pipeline(device),
            df_pipe: df_pipeline(device),
            comb_pipe: comb_pipeline(device),
        }
    }

    pub fn add_material(&mut self, device: &wgpu::Device, cmd_encoder: &mut wgpu::CommandEncoder, mat: &tobj::Material) {
        let mut specular = None;
        if !mat.specular_texture.is_empty() {
            let mut should_update = false;
            specular = Some(self.loaded_maps.get(&mat.specular_texture).map_or_else(
                || {
                    should_update = true;
                    load_texture(device, cmd_encoder, &mat.specular_texture)
                },
                |tv| { tv.clone() }
            ));

            if should_update {
                self.loaded_maps.insert(mat.specular_texture.clone(), specular.as_ref().unwrap().clone());
            }
        }

        let mut diffuse = None;
        if !mat.diffuse_texture.is_empty() {
            let mut should_update = false;
            diffuse = Some(self.loaded_maps.get(&mat.diffuse_texture).map_or_else(
                || {
                    should_update = true;
                    load_texture(device, cmd_encoder, &mat.diffuse_texture)
                },
                |tv| { tv.clone() }
            ));

            if should_update {
                self.loaded_maps.insert(mat.diffuse_texture.clone(), diffuse.as_ref().unwrap().clone());
            }
        }
        
        let new_mat = Material {
            mat_props: MaterialProps {
                metalness: 0.0,
                roughness: 0.0,
            },
            mat_maps: MaterialMaps {
                fixed_albedo: mat.diffuse,
                specular,
                diffuse,
            },
        };
        
        self.loaded_materials.insert(mat.name.clone(), new_mat);
    }

    fn activate_material<'a>(&'a self, mat_name: &str, rpass: &mut wgpu::RenderPass<'a>) {
        let mat = self.loaded_materials.get(mat_name).expect("unknown material was requested");
        
        if mat.mat_maps.specular.is_some() && mat.mat_maps.diffuse.is_some() {
            rpass.set_pipeline(&self.comb_pipe);
        } else if mat.mat_maps.specular.is_some() {
            rpass.set_pipeline(&self.sp_pipe);
        } else if mat.mat_maps.diffuse.is_some() {
            rpass.set_pipeline(&self.df_pipe);
        } else {
            rpass.set_pipeline(&self.fa_pipe);
        }
    }
}

fn fa_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let draw_call_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // projection view matrix data
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            // light data
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: Some("fa_pipe_draw_call_bg_layout")
    });

    let mesh_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // albedo
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: Some("fa_pipe_mesh_bg_layout")
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&draw_call_bg_layout, &mesh_bg_layout],
    });

    let vshader = include_bytes!("../../res/shaders/fixed_albedo.vert.spv");
    let vmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&vshader[..])).expect("failed to read fa_pipe vertex shader spir-v"));

    let fshader = include_bytes!("../../res/shaders/fixed_albedo.frag.spv");
    let fmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&fshader[..])).expect("failed to read fa_pipe fragment shader spir-v"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vmodule,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fmodule,
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
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: 24,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    // position
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 0,
                    },
                    // normal
                    wgpu::VertexAttributeDescriptor {
                        offset: 12,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 1,
                    },
                ],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

fn sp_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let draw_call_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // projection view matrix data
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            // sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    comparison: false,
                },
            },
            // light data
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: Some("sp_pipe_draw_call_bg_layout")
    });

    let mesh_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // specular map
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
        ],
        label: Some("sp_pipe_mesh_bg_layout")
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&draw_call_bg_layout, &mesh_bg_layout],
    });

    let vshader = include_bytes!("../../res/shaders/specular.vert.spv");
    let vmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&vshader[..])).expect("failed to read sp_pipe vertex shader spir-v"));

    let fshader = include_bytes!("../../res/shaders/specular.frag.spv");
    let fmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&fshader[..])).expect("failed to read sp_pipe fragment shader spir-v"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vmodule,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fmodule,
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
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: 36,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    // position
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 0,
                    },
                    // tex_coords
                    wgpu::VertexAttributeDescriptor {
                        offset: 12,
                        format: wgpu::VertexFormat::Float2,
                        shader_location: 1,
                    },
                    // normal
                    wgpu::VertexAttributeDescriptor {
                        offset: 20,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 2,
                    },
                ],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

fn df_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let draw_call_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // projection view matrix data
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            // sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    comparison: false,
                },
            },
            // light data
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: Some("df_pipe_draw_call_bg_layout"),
    });

    let mesh_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // diffuse map
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
        ],
        label: Some("df_pipe_mesh_bg_layout"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&draw_call_bg_layout, &mesh_bg_layout],
    });

    let vshader = include_bytes!("../../res/shaders/diffuse.vert.spv");
    let vmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&vshader[..])).expect("failed to read df_pipe vertex shader spir-v"));

    let fshader = include_bytes!("../../res/shaders/diffuse.frag.spv");
    let fmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&fshader[..])).expect("failed to read df_pipe fragment shader spir-v"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vmodule,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fmodule,
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
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: 36,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    // position
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 0,
                    },
                    // tex_coords
                    wgpu::VertexAttributeDescriptor {
                        offset: 12,
                        format: wgpu::VertexFormat::Float2,
                        shader_location: 1,
                    },
                    // normal
                    wgpu::VertexAttributeDescriptor {
                        offset: 20,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 2,
                    },
                ],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

fn comb_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
    let draw_call_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // projection view matrix data
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            // sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    comparison: false,
                },
            },
            // light data
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: Some("comb_pipe_draw_call_bg_layout"),
    });

    let mesh_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // diffuse map
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
            // specular map
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
        ],
        label: Some("comb_pipe_mesh_bg_layout"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&draw_call_bg_layout, &mesh_bg_layout],
    });

    let vshader = include_bytes!("../../res/shaders/combined.vert.spv");
    let vmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&vshader[..])).expect("failed to read comb_pipe vertex shader spir-v"));

    let fshader = include_bytes!("../../res/shaders/combined.frag.spv");
    let fmodule = device.create_shader_module(&wgpu::read_spirv(
        std::io::Cursor::new(&fshader[..])).expect("failed to read comb_pipe fragment shader spir-v"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vmodule,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fmodule,
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
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: 36,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    // position
                    wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 0,
                    },
                    // tex_coords
                    wgpu::VertexAttributeDescriptor {
                        offset: 12,
                        format: wgpu::VertexFormat::Float2,
                        shader_location: 1,
                    },
                    // normal
                    wgpu::VertexAttributeDescriptor {
                        offset: 20,
                        format: wgpu::VertexFormat::Float3,
                        shader_location: 2,
                    },
                ],
            }],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

fn load_texture(device: &wgpu::Device, cmd_encoder: &mut wgpu::CommandEncoder, texture_name: &str) -> Rc<wgpu::TextureView> {
    println!("[Info] Loading texture: {}", texture_name);
    let texture_image = {
        let mut image_file = File::open(format!("res/models/{}", texture_name)).expect("Failed to open texture image");
        let mut image_contents = vec![];
        let _ = image_file.read_to_end(&mut image_contents);
        
        let texture_image = image::load_from_memory(&image_contents)
            .or_else(|err| {
                if texture_name.ends_with(".tga") {
                    image::load_from_memory_with_format(&image_contents, image::ImageFormat::TGA)
                } else {
                    Err(err)
                }
            })
            .expect(&format!("failed to load a texture image: {}", texture_name));
        texture_image.into_rgba()
    };
    
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
        label: Some(texture_name),
    });

    let image_width = texture_image.width();
    let image_height = texture_image.height();
    let image_data = texture_image.into_vec();
    let image_buf = device.create_buffer_with_data(image_data.as_slice(), wgpu::BufferUsage::COPY_SRC);

    cmd_encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &image_buf,
            offset: 0,
            bytes_per_row: 4 * 4 * image_width, // four bytes per four floats per #of pixels
            rows_per_image: image_height,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::default(),
        },
        texture_extent
    );

    Rc::new(texture.create_default_view())
}
