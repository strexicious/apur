use winit::window::Window;
use glam::{Vec3};

mod model;
mod renderer;
mod material;
mod buffer;

use model::{Scene};
use renderer::{Renderer, camera::{Camera, Frustum}};
use material::MaterialManager;

#[inline]
fn angle_to_vec(angle: f32) -> Vec3 {
    Vec3::new(f32::cos(angle), f32::sin(angle), 0.0)
}

pub struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    renderer: Renderer,
    scene: Scene,
    mat_man: MaterialManager,
}

impl Engine {

    const CAMERA_SPEED: f32 = 5.0;
    
    pub fn new(window: &Window) -> Self {
        let window_width = window.inner_size().width;
        let window_height = window.inner_size().height;
        
        let surface = wgpu::Surface::create(window);
        let adapter = pollster::block_on(wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )).expect("Couldn't get hardware adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions::default(),
            limits: wgpu::Limits::default(),
        }));
        
        let surface = wgpu::Surface::create(window);
        let swapchain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window_width,
            height: window_height,
            present_mode: wgpu::PresentMode::Mailbox,
        });


        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("engine start") });
        
        let mut mat_man = MaterialManager::new(&device);
        let mut renderer = Renderer::new(
            &device,
            window_width,
            window_height,
            &mat_man,
        );
        
        let mut scene = Scene::default();
        scene.load_from_obj(&device, &mut encoder, "Pony_cartoon", &mut mat_man, &mut renderer);

        queue.submit(&[encoder.finish()]);
        
        Self {
            device,
            queue,
            swapchain,
            renderer,
            mat_man,
            scene,
        }
    }

    pub fn render(&mut self) {
        let frame = self.swapchain.get_next_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render") });

        self.renderer.render(&self.device, &frame, &mut encoder, &self.mat_man);
        self.queue.submit(&[encoder.finish()]);
    }

    pub fn handle_mouse_move(&mut self, dx: f64, dy: f64) {
        self.renderer.rotate_camera(dx as f32, dy as f32);
    }

    pub fn move_camera(&mut self, forward: bool) {
        self.renderer.move_camera(if forward { 1.0 } else { -1.0 } * Self::CAMERA_SPEED);
    }
}
