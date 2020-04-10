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
            fa_pipe: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    attributes: &[
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
                        wgpu::VertexAttributeDescriptor {
                            offset: 20,
                            format: wgpu::VertexFormat::Float3,
                            shader_location: 2,
                        },
                    ],
                }],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            }),
            sp_pipe: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    attributes: &[
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
                        wgpu::VertexAttributeDescriptor {
                            offset: 20,
                            format: wgpu::VertexFormat::Float3,
                            shader_location: 2,
                        },
                    ],
                }],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            }),
            df_pipe: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    attributes: &[
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
                        wgpu::VertexAttributeDescriptor {
                            offset: 20,
                            format: wgpu::VertexFormat::Float3,
                            shader_location: 2,
                        },
                    ],
                }],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            }),
            comb_pipe: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    attributes: &[
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
                        wgpu::VertexAttributeDescriptor {
                            offset: 20,
                            format: wgpu::VertexFormat::Float3,
                            shader_location: 2,
                        },
                    ],
                }],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            }),
        }
    }
}
