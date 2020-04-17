use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::collections::HashMap;

use super::buffer::ManagedBuffer;

pub enum Material {
    FA(FAMaterial),
    SP(SPMaterial),
    DF(DFMaterial),
    COMB(CombinedMaterial),
}

impl Material {
    pub fn get_mat_bg(&self) -> &wgpu::BindGroup {
        match self {
            Self::FA(mat) => &mat.mat_bg,
            Self::SP(mat) => &mat.mat_bg,
            Self::DF(mat) => &mat.mat_bg,
            Self::COMB(mat) => &mat.mat_bg,
        }
    }
}

#[derive(Default)]
struct MaterialProps {
    metalness: f32,
    roughness: f32,
}

pub struct FAMaterial {
    props: MaterialProps,
    albedo: ManagedBuffer,
    mat_bg: wgpu::BindGroup,
}

impl FAMaterial {
    fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, albedo: [f32; 3]) -> Self {
        let albedo_buf = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::UNIFORM, &albedo[..]);
        
        let mat_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: albedo_buf.get_buffer(),
                        range: 0 .. albedo_buf.get_size_bytes() as u64,
                    }
                }
            ],
            label: Some("FAMaterial bing-group"),
        });

        Self {
            mat_bg,
            albedo: albedo_buf,
            props: MaterialProps::default(),
        }
    }
}

impl FAMaterial {
    pub const SHADERS_SOURCE: (&'static [u8], &'static [u8]) = (
        include_bytes!("../../res/shaders/fixed_albedo.vert.spv"),
        include_bytes!("../../res/shaders/fixed_albedo.frag.spv")
    );
    
    pub const GLOBAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
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
        label: Some("fa_pipe_global_bg_layout"),
    };

    pub const MATERIAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // albedo
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
        ],
        label: Some("fa_pipe_mat_bg_layout")
    };

    pub const VERTEX_STATE: wgpu::VertexStateDescriptor<'static> = wgpu::VertexStateDescriptor {
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

pub struct SPMaterial {
    props: MaterialProps,
    albedo: ManagedBuffer,
    specular: Rc<wgpu::TextureView>,
    mat_bg: wgpu::BindGroup,
}

impl SPMaterial {
    fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, albedo: [f32; 3], specular: Rc<wgpu::TextureView>) -> Self {
        let albedo_buf = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::UNIFORM, &albedo[..]);
        
        let mat_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: albedo_buf.get_buffer(),
                        range: 0 .. albedo_buf.get_size_bytes() as u64,
                    }
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&*specular),
                },
            ],
            label: None,
        });
        
        Self {
            mat_bg,
            specular,
            albedo: albedo_buf,
            props: MaterialProps::default(),
        }
    }
}

impl SPMaterial {
    pub const SHADERS_SOURCE: (&'static [u8], &'static [u8]) = (
        include_bytes!("../../res/shaders/specular.vert.spv"),
        include_bytes!("../../res/shaders/specular.frag.spv")
    );
    
    pub const GLOBAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
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
        label: Some("sp_pipe_global_bg_layout"),
    };

    pub const MATERIAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // albedo
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
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
        label: Some("sp_pipe_mat_bg_layout")
    };

    pub const VERTEX_STATE: wgpu::VertexStateDescriptor<'static> = wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: 32,
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
    };
}

pub struct DFMaterial {
    props: MaterialProps,
    albedo: ManagedBuffer,
    diffuse: Rc<wgpu::TextureView>,
    mat_bg: wgpu::BindGroup,
}

impl DFMaterial {
    fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, albedo: [f32; 3], diffuse: Rc<wgpu::TextureView>) -> Self {
        let albedo_buf = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::UNIFORM, &albedo[..]);
        
        let mat_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: albedo_buf.get_buffer(),
                        range: 0 .. albedo_buf.get_size_bytes() as u64,
                    }
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&*diffuse),
                },
            ],
            label: None,
        });
        
        Self {
            mat_bg,
            diffuse,
            albedo: albedo_buf,
            props: MaterialProps::default(),
        }
    }
}

impl DFMaterial {
    pub const SHADERS_SOURCE: (&'static [u8], &'static [u8]) = (
        include_bytes!("../../res/shaders/diffuse.vert.spv"),
        include_bytes!("../../res/shaders/diffuse.frag.spv")
    );
    
    pub const GLOBAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
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
        label: Some("df_pipe_global_bg_layout"),
    };

    pub const MATERIAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // albedo
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            },
            // diffuse map
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
        label: Some("df_pipe_mat_bg_layout")
    };

    pub const VERTEX_STATE: wgpu::VertexStateDescriptor<'static> = wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: 32,
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
    };
}

pub struct CombinedMaterial {
    props: MaterialProps,
    albedo: ManagedBuffer,
    specular: Rc<wgpu::TextureView>,
    diffuse: Rc<wgpu::TextureView>,
    mat_bg: wgpu::BindGroup,
}

impl CombinedMaterial {
    fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, albedo: [f32; 3], specular: Rc<wgpu::TextureView>, diffuse: Rc<wgpu::TextureView>) -> Self {
        let albedo_buf = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::UNIFORM, &albedo[..]);
        
        let mat_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: albedo_buf.get_buffer(),
                        range: 0 .. albedo_buf.get_size_bytes() as u64,
                    }
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&*diffuse),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&*specular),
                },
            ],
            label: None,
        });
        
        Self {
            mat_bg,
            diffuse,
            specular,
            albedo: albedo_buf,
            props: MaterialProps::default(),
        }
    }
}

impl CombinedMaterial {
    pub const SHADERS_SOURCE: (&'static [u8], &'static [u8]) = (
        include_bytes!("../../res/shaders/combined.vert.spv"),
        include_bytes!("../../res/shaders/combined.frag.spv")
    );
    
    pub const GLOBAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
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
        label: Some("comb_pipe_global_bg_layout"),
    };

    pub const MATERIAL_BG_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        bindings: &[
            // albedo
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
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
            // diffuse map
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            },
        ],
        label: Some("comb_pipe_mat_bg_layout")
    };

    pub const VERTEX_STATE: wgpu::VertexStateDescriptor<'static> = wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: 32,
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
    };
}

pub struct MaterialManager {
    loaded_maps: HashMap<String, Rc<wgpu::TextureView>>,
    loaded_materials: HashMap<String, Material>,

    fa_mat_bg_lay: wgpu::BindGroupLayout,
    sp_mat_bg_lay: wgpu::BindGroupLayout,
    df_mat_bg_lay: wgpu::BindGroupLayout,
    comb_mat_bg_lay: wgpu::BindGroupLayout,
}

impl MaterialManager {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            loaded_maps: HashMap::new(),
            loaded_materials: HashMap::new(),
            fa_mat_bg_lay: device.create_bind_group_layout(&FAMaterial::MATERIAL_BG_LAYOUT),
            sp_mat_bg_lay: device.create_bind_group_layout(&SPMaterial::MATERIAL_BG_LAYOUT),
            df_mat_bg_lay: device.create_bind_group_layout(&DFMaterial::MATERIAL_BG_LAYOUT),
            comb_mat_bg_lay: device.create_bind_group_layout(&CombinedMaterial::MATERIAL_BG_LAYOUT),
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
        
        let matt = match (specular, diffuse) {
            (None, None) => Material::FA(FAMaterial::new(device, &self.fa_mat_bg_lay, mat.diffuse)),
            (Some(tvsp), None) => Material::SP(SPMaterial::new(device, &self.sp_mat_bg_lay, mat.diffuse, tvsp)),
            (None, Some(tvdf)) => Material::DF(DFMaterial::new(device, &self.df_mat_bg_lay, mat.diffuse, tvdf)),
            (Some(tvsp), Some(tvdf)) => Material::COMB(CombinedMaterial::new(device, &self.comb_mat_bg_lay, mat.diffuse, tvsp, tvdf)),
        };

        self.loaded_materials.insert(mat.name.clone(), matt);
    }

    pub fn get_material(&self, name: &str) -> Option<&Material> {
        self.loaded_materials.get(name)
    }

    pub fn fa_mat_bg_layout(&self) -> &wgpu::BindGroupLayout {
        &self.fa_mat_bg_lay
    }

    pub fn sp_mat_bg_layout(&self) -> &wgpu::BindGroupLayout {
        &self.sp_mat_bg_lay
    }

    pub fn df_mat_bg_layout(&self) -> &wgpu::BindGroupLayout {
        &self.df_mat_bg_lay
    }

    pub fn comb_mat_bg_layout(&self) -> &wgpu::BindGroupLayout {
        &self.comb_mat_bg_lay
    }
}

fn load_texture(device: &wgpu::Device, cmd_encoder: &mut wgpu::CommandEncoder, texture_name: &str) -> Rc<wgpu::TextureView> {
    println!("[Info] Loading texture: {}", texture_name);
    
    let texture_image = {
        let mut image_file = File::open(format!("res/models/{}", texture_name)).expect("Failed to open texture image");
        let mut image_contents = vec![];
        let _ = image_file.read_to_end(&mut image_contents);
        
        let texture_image = if texture_name.ends_with(".tga") {
            image::load_from_memory_with_format(&image_contents, image::ImageFormat::TGA)
        } else {
            image::load_from_memory(&image_contents)
        }.expect(&format!("failed to load a texture image: {}", texture_name));
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
            bytes_per_row: 4 * image_width, // four bytes per four floats per #of pixels
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
