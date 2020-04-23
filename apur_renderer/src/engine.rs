use winit::window::Window;
use winit::event::KeyboardInput;

use super::renderer::SolidRenderer;
use super::world::World;
use super::texture_manager::TextureManager;
use super::buffer::ManagedBuffer;

pub struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    ds_texture: wgpu::TextureView,
    transforms_buffer: ManagedBuffer,
    lights_buffer: ManagedBuffer,
    world: World,
    solid_rdr: SolidRenderer,
    tex_man: TextureManager,
}

impl Engine {
    pub fn new(window: &Window) -> Self {
        let window_width = window.inner_size().width;
        let window_height = window.inner_size().height;
        let mut world = World::new(window_width, window_height);
        
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
        
        let swapchain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window_width,
            height: window_height,
            present_mode: wgpu::PresentMode::Mailbox,
        });

        let cam = world.get_camera();
        let transforms = [cam.view().to_cols_array(), cam.projection().to_cols_array()];
        let transforms_buffer = ManagedBuffer::from_data(&device, wgpu::BufferUsage::UNIFORM, &transforms);
        
        let lights = world.get_lights().iter().map(|l| l.to_shader_data()).flatten().collect::<Vec<f32>>();
        let lights_buffer = ManagedBuffer::from_data(&device, wgpu::BufferUsage::UNIFORM, &lights);

        let solid_rdr = SolidRenderer::new(&device, &transforms_buffer, &lights_buffer);
        let tex_man = TextureManager::new(&device);
    
        let ds_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width: window_width, height: window_height, depth: 1, },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("Depth-Stencil texture"),
        }).create_default_view();

        Self {
            device,
            queue,
            swapchain,
            ds_texture,
            transforms_buffer,
            lights_buffer,
            solid_rdr,
            world,
            tex_man,
        }
    }

    pub fn render(&mut self) {
        let frame = self.swapchain.get_next_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render") });

        const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.2, g: 0.5, b: 0.7, a: 1.0 };
        
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            self.solid_rdr.render(&mut rpass, self.world.get_solid_objects());
        }

        self.queue.submit(&[encoder.finish()]);
    }

    pub fn handle_mouse_move(&mut self, dx: f64, dy: f64) {
        // self.renderer.rotate_camera(dx as f32, dy as f32);
    }

    pub fn handle_key_input(&mut self, input: KeyboardInput) {
        // self.renderer.move_camera(if forward { 1.0 } else { -1.0 } * Self::CAMERA_SPEED);
    }
}
