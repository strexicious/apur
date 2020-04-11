pub mod camera;
pub mod light;

use camera::{Camera, Frustum};
use light::{Light};

use super::model::Scene;
use super::material::MaterialManager;

pub struct Renderer {
    ds_texture: wgpu::TextureView,
    camera: Camera,
    frustum: Frustum,
    lights: Vec<Light>,
    // environment: Environment,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
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
        
        Self {
            ds_texture: depth_texture.create_default_view(),
            camera: Camera::default(),
            frustum: Frustum::new(width, height),
            lights: vec![],
        }
    }

    pub fn render(
        &self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
        cmd_encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
        mat_man: &MaterialManager,
    ) {
        // material can be an enum
        // foreach material in material manager:
        //   material.activate_pipeline()
        //   scene.draw_objects(material)
    }
}
