use winit::window::Window;
use winit::event::DeviceEvent;

mod camera;
mod model;
mod renderer;

use model::{Model};
use camera::{Camera, Frustum};
use renderer::{Renderer, RenderData};

pub struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    depth_texture_view: wgpu::TextureView,
    render_data: RenderData,
    renderer: Renderer,
    camera: Camera,
    frustum: Frustum,
}

impl Engine {
    pub fn new(window: &Window) -> Self {
        let window_width = window.inner_size().width;
        let window_height = window.inner_size().height;
        
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,
        }).expect("Couldn't get hardware adapter");
        let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions::default(),
            limits: wgpu::Limits::default(),
        });
        
        let surface = wgpu::Surface::create(window);
        let swapchain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window_width,
            height: window_height,
            present_mode: wgpu::PresentMode::Vsync,
        });

        let mut camera = Camera::default();
        camera.move_pos(-5.0);
        let frustum = Frustum::new(window_width, window_height);
        
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width: window_width, height: window_height, depth: 1, },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let depth_texture_view = depth_texture.create_default_view();

        let renderer = Renderer::new(&device);
        let model = Model::load_model(&device, &mut queue, "Planks of wood");
        let render_data = RenderData::new(
            &device,
            model,
            camera.view(),
            frustum.projection(),
            renderer.get_bind_group_layout(),
            renderer.get_texture_bind_group_layout()
        );

        Self { device, queue, swapchain, depth_texture_view, render_data, renderer, camera, frustum }
    }

    pub fn render(&mut self) {
        let frame = self.swapchain.get_next_texture();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.renderer.render(&frame, &mut encoder, &self.depth_texture_view, &self.render_data);
        self.queue.submit(&[encoder.finish()]);
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        #[allow(clippy::single_match)]
        match event {
            DeviceEvent::MouseMotion{ delta: (dx, dy) } => {
                self.camera.change_angle(dx as f32, dy as f32);
                self.render_data.update_view(self.camera.view());
            },
            _ => { },
        }
    }
}
