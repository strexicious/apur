use std::rc::Rc;
use std::collections::HashMap;

struct MaterialProps {
    metalness: f32,
    roughness: f32,
}

struct MaterialMaps {
    fixed_albedo: (f32, f32, f32),
    specular: Option<Rc<wgpu::TextureView>>,
    diffuse: Option<Rc<wgpu::TextureView>>,
}

pub struct Material {
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
    fn new(device: &wgpu::Device) -> Self {
        Self {
            loaded_maps: HashMap::new(),
            loaded_materials: HashMap::new(),
            fa_pipe: device.create_render_pipeline(&fa_pipe_descriptor(device)),
            sp_pipe: device.create_render_pipeline(&sp_pipe_descriptor(device)),
            df_pipe: device.create_render_pipeline(&df_pipe_descriptor(device)),
            comb_pipe: device.create_render_pipeline(&comb_pipe_descriptor(device)),
        }
    }
}

fn fa_pipe_descriptor(device: &wgpu::Device) -> wgpu::RenderPipelineDescriptor {
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

    wgpu::RenderPipelineDescriptor {
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
    }
}

fn sp_pipe_descriptor(device: &wgpu::Device) -> wgpu::RenderPipelineDescriptor {
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

    wgpu::RenderPipelineDescriptor {
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
    }
}

fn df_pipe_descriptor(device: &wgpu::Device) -> wgpu::RenderPipelineDescriptor {
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

    wgpu::RenderPipelineDescriptor {
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
    }
}

fn comb_pipe_descriptor(device: &wgpu::Device) -> wgpu::RenderPipelineDescriptor {
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

    wgpu::RenderPipelineDescriptor {
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
    }
}
