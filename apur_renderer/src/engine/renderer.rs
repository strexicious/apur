pub mod camera;
pub mod light;
mod pipeline;

use camera::{Camera, Frustum};
use light::{Light};
use pipeline::Pipeline;

use super::buffer::ManagedBuffer;
use super::model::{Mesh};
use super::material::{MaterialManager, Material, FAMaterial, SPMaterial, DFMaterial, CombinedMaterial};

pub struct Renderer {
    ds_texture: wgpu::TextureView,
    update_transforms: bool,
    transforms: ManagedBuffer,
    lights_buf: ManagedBuffer,
    sampler: wgpu::Sampler,
    camera: Camera,
    frustum: Frustum,
    light: Light,
    // environment: Environment,

    fa_pipe: Pipeline,
    sp_pipe: Pipeline,
    df_pipe: Pipeline,
    comb_pipe: Pipeline,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        mat_man: &MaterialManager,
    ) -> Self {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width, height, depth: 1, },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("Depth-Stencil texture"),
        });
        
        let camera = Camera::default();
        let frustum = Frustum::new(width, height);
        let mut transforms_data = vec![];
        transforms_data.extend(camera.view().as_ref());
        transforms_data.extend(frustum.projection().as_ref());

        let transforms = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_DST, &transforms_data);
        let lights_buf = ManagedBuffer::from_f32_data(device, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_DST, &[0f32, -1f32, 0f32]);

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

        let gb1 = &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: transforms.get_buffer(),
                    range: 0 .. 2 * 64,
                },
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::Buffer {
                    buffer: lights_buf.get_buffer(),
                    range: 0 .. 12,
                },
            },
        ];

        let gb2 = &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: transforms.get_buffer(),
                    range: 0 .. 2 * 64,
                },
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::Binding {
                binding: 2,
                resource: wgpu::BindingResource::Buffer {
                    buffer: lights_buf.get_buffer(),
                    range: 0 .. 12,
                },
            },
        ];
        
        Self {
            camera,
            frustum,
            update_transforms: false,
            ds_texture: depth_texture.create_default_view(),
            light: Light::Directional(Default::default()),

            fa_pipe: Pipeline::new(
                device,
                FAMaterial::SHADERS_SOURCE,
                &FAMaterial::GLOBAL_BG_LAYOUT,
                gb1,
                mat_man.fa_mat_bg_layout(),
                &FAMaterial::VERTEX_STATE,
            ),
            sp_pipe: Pipeline::new(
                device,
                SPMaterial::SHADERS_SOURCE,
                &SPMaterial::GLOBAL_BG_LAYOUT,
                gb2,
                mat_man.sp_mat_bg_layout(),
                &SPMaterial::VERTEX_STATE,
            ),
            df_pipe: Pipeline::new(
                device,
                DFMaterial::SHADERS_SOURCE,
                &DFMaterial::GLOBAL_BG_LAYOUT,
                gb2,
                mat_man.df_mat_bg_layout(),
                &DFMaterial::VERTEX_STATE,
            ),
            comb_pipe: Pipeline::new(
                device,
                CombinedMaterial::SHADERS_SOURCE,
                &CombinedMaterial::GLOBAL_BG_LAYOUT,
                gb2,
                mat_man.comb_mat_bg_layout(),
                &CombinedMaterial::VERTEX_STATE,
            ),

            transforms,
            lights_buf,
            sampler,
        }
    }

    pub fn add_meshes(&mut self, meshes: Vec<Mesh>, mat_manager: &MaterialManager,) {
        for m in meshes {
            let mat = mat_manager.get_material(m.get_mat_name()).unwrap();
            match mat {
                Material::FA(_) => {
                    self.fa_pipe.add_mesh(m);
                },
                Material::SP(_) => {
                    self.sp_pipe.add_mesh(m);
                },
                Material::DF(_) => {
                    self.df_pipe.add_mesh(m);
                },
                Material::COMB(_) => {
                    self.comb_pipe.add_mesh(m);
                },
            }
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        frame: &wgpu::SwapChainOutput,
        cmd_encoder: &mut wgpu::CommandEncoder,
        mat_man: &MaterialManager,
    ) {
        const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.2, g: 0.5, b: 0.7, a: 1.0 };

        if self.update_transforms {
            self.update_transforms = false;
            self.transforms.update_f32_data(device, cmd_encoder, 0, self.camera.view().as_ref());
        }
        
        let mut rpass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: CLEAR_COLOR,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.ds_texture,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        self.fa_pipe.draw_meshes(&mut rpass, &mat_man);
        self.sp_pipe.draw_meshes(&mut rpass, &mat_man);
        self.df_pipe.draw_meshes(&mut rpass, &mat_man);
        self.comb_pipe.draw_meshes(&mut rpass, &mat_man);
    }

    pub fn rotate_camera(&mut self, dx: f32, dy: f32) {
        self.camera.change_angle(dx, dy);
        self.update_transforms = true;
    }

    pub fn move_camera(&mut self, factor: f32) {
        self.camera.move_pos(factor);
        self.update_transforms = true;
    }
}