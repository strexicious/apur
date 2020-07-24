use log::info;
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::error as apur_error;
use super::event_handler::EventHandler;

/// Something that drives the execution of an application, rendering on screen,
/// handling events and updating during that time.
pub trait ApplicationDriver: 'static {
    fn current_event_handler(&mut self, app: &mut Application) -> Option<&mut dyn EventHandler>;
    fn update(&mut self, app: &mut Application) -> Vec<wgpu::CommandBuffer>;
    fn render(
        &mut self,
        app: &mut Application,
        frame: &wgpu::SwapChainOutput,
    ) -> Vec<wgpu::CommandBuffer>;
}

pub struct Application {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    swapchain: wgpu::SwapChain,
    queue: wgpu::Queue,
    device: wgpu::Device,
}

impl Application {
    pub async fn new<S: Into<String>>(
        title: S,
        width: u16,
        height: u16,
    ) -> apur_error::Result<Self> {
        // we wrap the event_loop into an option because
        // the option is consumed when the event_loop is ran,
        // and we leave the rest of the Application intact.
        let event_loop = Some(EventLoop::new());
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height))
            .with_resizable(false)
            .build(event_loop.as_ref().unwrap())
            .expect("Error building window");

        let surface = wgpu::Surface::create(&window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .expect("Couldn't get hardware adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions::default(),
                limits: wgpu::Limits::default(),
            })
            .await;
        let swapchain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: width as u32,
                height: height as u32,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );

        Ok(Self {
            window,
            event_loop,
            swapchain,
            queue,
            device,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Executes the main event loop which polls events for the owned window.
    /// Mouse and keyboard events are passed to the [`EventHandler`] in use from
    /// the [`ApplicationDriver`]. It also provides behaviour for updating
    /// and rendering for the application.
    ///
    /// [`EventHandler`]: ../event_handler/trait.EventHandler.html
    /// [`ApplicationDriver`]: trait.ApplicationDriver.html
    pub fn run(mut self, mut driver: impl ApplicationDriver) -> ! {
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow: &mut ControlFlow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event,
                    window_id: _,
                } => match event {
                    WindowEvent::CloseRequested => {
                        info!("Shutting down...");
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(event_handler) = driver.current_event_handler(&mut self) {
                            event_handler.handle_key(input);
                        }
                    }
                    _ => {}
                },
                Event::DeviceEvent {
                    event,
                    device_id: _,
                } => match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        if let Some(event_handler) = driver.current_event_handler(&mut self) {
                            event_handler.handle_mouse_move(dx as f32, dy as f32);
                        }
                    }
                    _ => {}
                },
                Event::MainEventsCleared => {
                    let cmd_buffers = driver.update(&mut self);
                    self.queue.submit(&cmd_buffers);
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let next_frame = self
                        .swapchain
                        .get_next_texture()
                        .expect("Failed to get next frame from swapchain");
                    let cmd_buffers = driver.render(&mut self, &next_frame);
                    self.queue.submit(&cmd_buffers);
                }
                _ => {}
            }
        })
    }
}
