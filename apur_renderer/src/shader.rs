pub trait Shader {
    const GLOBAL_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static>;
    const ELEMENT_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static>;
    const VERTEX_MODULE: &'static [u8];
    const FRAGMENT_MODULE: &'static [u8];
    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static>;
}

pub struct SolidShader;

impl Shader for SolidShader {
    const GLOBAL_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
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
        label: Some("SolidShader GLOBAL_LAYOUT_DESC"),
    };

    const ELEMENT_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // albedo, roughness
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: Some("SolidShader ELEMENT_LAYOUT_DESC")
    };

    const VERTEX_MODULE: &'static [u8] = include_bytes!("../res/shaders/solid.vert.spv");
    const FRAGMENT_MODULE: &'static [u8] = include_bytes!("../res/shaders/solid.frag.spv");

    const VERTEX_STATE_DESC: wgpu::VertexStateDescriptor<'static> = wgpu::VertexStateDescriptor {
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
    };
}
