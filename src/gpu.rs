use wgpu::{include_wgsl, util::DeviceExt, Extent3d};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use pollster::block_on;

pub struct Gpu {
    pub surface : wgpu::Surface,
    pub device : wgpu::Device,
    pub queue : wgpu::Queue,
    pub hidpi_factor : f64,
    pub surface_desc : wgpu::SurfaceConfiguration,
}

impl Gpu {
    pub fn new(window : &mut Window) -> Gpu {

        env_logger::init();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends : wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let version = env!("CARGO_PKG_VERSION");

        window.set_inner_size(LogicalSize {
            width : 1280.0,
            height : 720.0,
        });

        window.set_title(&format!("imgui-wgpu {version}"));

        let size = window.inner_size();

        let surface = unsafe {

            instance.create_surface(window)
        }
        .unwrap();

        let hidpi_factor = window.scale_factor();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference : wgpu::PowerPreference::HighPerformance,
            compatible_surface : Some(&surface),
            force_fallback_adapter : false,
        }))
        .unwrap();

        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        // Set up swap chain
        let surface_desc = wgpu::SurfaceConfiguration {
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
            format : wgpu::TextureFormat::Bgra8UnormSrgb,
            width : size.width,
            height : size.height,
            present_mode : wgpu::PresentMode::Fifo,
            alpha_mode : wgpu::CompositeAlphaMode::Auto,
            view_formats : vec![wgpu::TextureFormat::Bgra8Unorm],
        };

        surface.configure(&device, &surface_desc);

        Gpu {
            surface,
            device,
            queue,
            hidpi_factor,
            surface_desc,
        }
    }
}
