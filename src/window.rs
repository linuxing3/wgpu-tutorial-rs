use winit::{dpi::LogicalSize, event_loop::EventLoop, window::Window};

pub struct WindowController {
    window : Box<Window>,
}

impl WindowController {
    pub fn new(event_loop : EventLoop<()>) -> Self {

        let version = env!("CARGO_PKG_VERSION");

        let window = Box::new(Window::new(&event_loop).unwrap());

        window.set_inner_size(LogicalSize {
            width : 1280.0,
            height : 720.0,
        });

        window.set_title(&format!("imgui-wgpu {version}"));

        Self { window }
    }

    pub fn window(&self) -> &Window { self.window.as_ref() }
}
