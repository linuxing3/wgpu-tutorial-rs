use imgui::*;

pub struct ImguiController {
    pub imgui_context : imgui::Context,
    pub platform : imgui_winit_support::WinitPlatform,
}

impl ImguiController {
    pub fn new(window : &winit::window::Window, hidpi_factor : f64) -> ImguiController {

        // Set up dear imgui
        let mut imgui_context = imgui::Context::create();

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

        platform.attach_window(
            imgui_context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        imgui_context.set_ini_filename(None);

        let font_size = (13.0 * hidpi_factor) as f32;

        imgui_context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui_context
            .fonts()
            .add_font(&[FontSource::DefaultFontData {
                config : Some(imgui::FontConfig {
                    oversample_h : 1,
                    pixel_snap_h : true,
                    size_pixels : font_size,
                    ..Default::default()
                }),
            }]);

        ImguiController {
            imgui_context,
            platform,
        }
    }

    pub fn imgui_context(&self) -> &imgui::Context { &self.imgui_context }

    pub fn platform(&self) -> &imgui_winit_support::WinitPlatform { &self.platform }
}