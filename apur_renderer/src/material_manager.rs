use std::io::Read;
use std::fs::File;
use std::collections::HashMap;

use super::buffer::ManagedBuffer;

#[derive(Default)]
pub struct MaterialManager {
    textures: HashMap<String, wgpu::TextureView>,
    buffers: HashMap<String, ManagedBuffer>,
}

impl MaterialManager {
    pub fn load_texture(&mut self, device: &wgpu::Device, cmd_encoder: &mut wgpu::CommandEncoder, texture_name: String) {
        println!("[Info] Loading texture: {}", texture_name);
        
        let texture_image = {
            let mut image_file = File::open(format!("res/models/{}", &texture_name)).expect("Failed to open texture image");
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
            label: Some(&texture_name),
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

        self.textures.insert(texture_name, texture.create_default_view());
    }
    
    pub fn add_buffer(&mut self, mat_name: String, buffer: ManagedBuffer) {
        self.buffers.insert(mat_name, buffer);
    }

    pub fn get_texture(&self, texture_name: &str) -> &wgpu::TextureView {
        self.textures.get(texture_name).unwrap()
    }

    pub fn get_buffer(&self, mat_name: &str) -> &ManagedBuffer {
        self.buffers.get(mat_name).unwrap()
    }
}
