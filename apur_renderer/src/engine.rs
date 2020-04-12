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

    const CAMERA_SPEED: f32 = 0.5;
    
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
        let (device, mut queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
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
        scene.load_from_obj(&device, &mut encoder, "sponza", &mut mat_man, &mut renderer);

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
        
        // TODO: maybe get RenderData::update_view working...
        // ... ok so after a little backtracking and talking, update_view sets a write pending,
        // queue.submit fails because there is a write pending, and currently wgpu doesn't
        // have auto flushing or sync with write and queue submission, so we would have to manually
        // wait till the write callback in RenderData::update_view is called
        // the devs said the api isn't finalized yet and there are some other proposed changes
        // that may land soon, keep following them.
        // the devs also said APIs *could* store buffer pools internally so allocating temp buffers
        // for copying may not even be expensive
        // ...but these pools are not guarenteed tho so idk man
        // https://github.com/gfx-rs/wgpu-rs/issues/9#issuecomment-494022784
        // https://github.com/gpuweb/gpuweb/pull/509
        // self.render_data.update_view(self.camera.view());
        // if self.update_mats {
        //     self.update_mats = false;
        //     let temp_buffer = self.device
        //         .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
        //         .fill_from_slice(&[self.camera.view()]);
        //     encoder.copy_buffer_to_buffer(&temp_buffer, 0, self.render_data.get_uniforms_buffer(), 0, 64);
        // }

        // if self.update_light {
        //     self.update_light = false;
        //     let temp_buffer = self.device
        //         .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
        //         .fill_from_slice(&[angle_to_vec(self.light_dir_angle)]);
        //     encoder.copy_buffer_to_buffer(&temp_buffer, 0, self.render_data.get_light_ubo(), 0, 16);
        // }

        self.renderer.render(&frame, &mut encoder, &self.mat_man);
        self.queue.submit(&[encoder.finish()]);
    }

    pub fn handle_mouse_move(&mut self, dx: f64, dy: f64) {
        // self.camera.change_angle(dx as f32, dy as f32);
        // self.update_mats = true;
    }

    pub fn move_camera(&mut self, forward: bool) {
        // self.camera.move_pos(if forward { 1.0 } else { -1.0 } * Self::CAMERA_SPEED);
        // self.update_mats = true;
    }

    pub fn rotate_light(&mut self, right: bool) {
        // if right {
        //     self.light_dir_angle += f32::to_radians(1.0);
        // } else {
        //     self.light_dir_angle -= f32::to_radians(1.0);
        // }
        // self.update_light = true;
    }
}
