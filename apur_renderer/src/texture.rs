pub trait Texture {
    fn view(&self) -> &wgpu::TextureView;
}

pub trait CopyableTexture {
    fn copy_view(&mut self) -> &mut wgpu::TextureCopyView;
}

pub struct DepthTexture {
    texture: wgpu::Texture,
    default_view: wgpu::TextureView,
}

impl DepthTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("Depth texture"),
        });

        let default_view = texture.create_default_view();

        Self {
            texture,
            default_view,
        }
    }
}

impl Texture for DepthTexture {
    fn view(&self) -> &wgpu::TextureView {
        &self.default_view
    }
}

pub struct DepthStencilTexture {
    texture: wgpu::Texture,
    default_view: wgpu::TextureView,
}

impl DepthStencilTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("Depth-Stencil texture"),
        });

        let default_view = texture.create_default_view();

        Self {
            texture,
            default_view,
        }
    }
}

impl Texture for DepthStencilTexture {
    fn view(&self) -> &wgpu::TextureView {
        &self.default_view
    }
}

pub struct FragmentOutputTexture {
    texture: wgpu::Texture,
    default_view: wgpu::TextureView,
}

impl FragmentOutputTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("rgba32 out"),
        });

        let default_view = texture.create_default_view();

        Self {
            texture,
            default_view,
        }
    }
}

impl Texture for FragmentOutputTexture {
    fn view(&self) -> &wgpu::TextureView {
        &self.default_view
    }
}

pub struct Sampler {
    sampler: wgpu::Sampler,
}

impl Sampler {
    pub fn new_general(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        Self { sampler }
    }
}

impl AsRef<wgpu::Sampler> for Sampler {
    fn as_ref(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
