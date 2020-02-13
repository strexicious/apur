use std::io::Read;
use std::fs::File;

#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 3],
    tex_coords: [f32; 2],
}

pub struct Mesh {
    indices_offset: u32,
    indices_count: u32,
    texture_view: wgpu::TextureView,
}

impl Mesh {    
    pub fn get_texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn get_indices_offset(&self) -> u32 {
        self.indices_offset
    }

    pub fn get_indices_count(&self) -> u32 {
        self.indices_count
    }
}

pub struct Model {
    vertex_buffer: wgpu::Buffer,
    indices_buffer: wgpu::Buffer,
    meshes: Vec<Mesh>,
}

impl Model {
    pub fn load_model(
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        obj_filename: &str,
    ) -> Self {
        let (models, mats) = tobj::load_obj(format!("res/models/{}.obj", obj_filename).as_ref()).expect("Failed to load the model");
        
        let mut cmd_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        
        let textured_models = models
            .into_iter()
            .filter(|m| {
                let mat_idx = m.mesh.material_id.expect("no material associated");
                !mats[mat_idx].diffuse_texture.is_empty()
            });
        
        let mut indices = vec![];
        let mut vertices = vec![];
        let mut meshes = vec![];
        
        for m in textured_models {
            let vs = m.mesh.positions;
            let ts = m.mesh.texcoords;

            assert_eq!(vs.len() / 3, ts.len() / 2, "positions and texcoords length not same");

            let vertices_len = vertices.len();
            vertices.extend(vs
                .chunks(3)
                .zip(ts.chunks(2))
                .map(|(vs, ts)| Vertex {
                    pos: [vs[0], vs[1], vs[2]],
                    tex_coords: [ts[0], ts[1]],
                })
            );

            let indices_offset = indices.len() as u32;
            let indices_count = m.mesh.indices.len() as u32;
            
            indices.extend(m.mesh.indices.into_iter().map(|idx| idx + vertices_len as u32));
            
            let mat_idx = m.mesh.material_id.expect("no material associated");
            assert!(!mats[mat_idx].diffuse_texture.is_empty(), "diffuse texture path empty");
            
            // assumed texture_name includes the "texture/" in path name
            let texture_name = &mats[mat_idx].diffuse_texture;
            println!("[Info] Loading texture: {}", texture_name);
            let texture_image = {
                let mut image_file = File::open(format!("res/models/{}", texture_name)).expect("Failed to open texture image");
                let mut image_contents = vec![];
                let _ = image_file.read_to_end(&mut image_contents);
                
                let texture_image = image::load_from_memory(&image_contents)
                    .expect("failed to load a texture image");
                texture_image.into_rgba()
            };
            
            let texture_extent = wgpu::Extent3d {
                width: texture_image.width(),
                height: texture_image.height(),
                depth: 1,
            };
            
            let texture_view = {
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

                texture.create_default_view()
            };
        
            meshes.push(Mesh {
                indices_offset,
                indices_count,
                texture_view,
            });
        }

        queue.submit(&[cmd_encoder.finish()]);

        let indices_buffer = device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices);
        let vertex_buffer = device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);
        
        Self { indices_buffer, vertex_buffer, meshes }
    }

    pub fn get_indices_buffer(&self) -> &wgpu::Buffer {
        &self.indices_buffer
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn get_meshes(&self) -> &[Mesh] {
        &self.meshes
    }
}