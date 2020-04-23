use winit::window::Window;

use super::renderer::SolidRenderer;
use super::world::World;
use super::material_manager::MaterialManager;

pub struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain: wgpu::SwapChain,
    world: World,
    solid_rdr: SolidRenderer,
    mat_man: MaterialManager,
}

impl Engine {
    pub fn new(window: &Window, world: World) -> Self {
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
        
        let swapchain = device.create_swap_chain(&surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window_width,
            height: window_height,
            present_mode: wgpu::PresentMode::Mailbox,
        });

        let solid_rdr = SolidRenderer::new(&device);
        let mat_man = MaterialManager::default();

        Self {
            device,
            queue,
            swapchain,
            solid_rdr,
            world,
            mat_man,
        }
    }

    pub fn render(&mut self) {
        let frame = self.swapchain.get_next_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render") });

        // self.solid_rdr.render(self.world.get_solid_objects());
        self.queue.submit(&[encoder.finish()]);
    }

    pub fn handle_mouse_move(&mut self, dx: f64, dy: f64) {
        // self.renderer.rotate_camera(dx as f32, dy as f32);
    }

    pub fn move_camera(&mut self, forward: bool) {
        // self.renderer.move_camera(if forward { 1.0 } else { -1.0 } * Self::CAMERA_SPEED);
    }
}

/*

    ds_texture: wgpu::TextureView,
    transforms: ManagedBuffer,
    lights_buf: ManagedBuffer,

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

    */
