use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};

pub struct ImguiController {
    pub imgui_context : Option<imgui::Context>,
    pub platform : Option<imgui_winit_support::WinitPlatform>,
}

impl ImguiController {
    // pub fn create_render(
    //     &mut self,
    //     surface_desc : wgpu::SurfaceConfiguration,
    //     device : &wgpu::Device,
    //     queue : &wgpu::Queue,
    // ) -> Renderer {
    //
    //     let renderer_config = RendererConfig {
    //         texture_format : surface_desc.format,
    //         ..Default::default()
    //     };
    //
    //     Renderer::new(
    //         &mut self.imgui_context.unwrap(),
    //         &device,
    //         &queue,
    //         renderer_config,
    //     )
    // }

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
            imgui_context : Some(imgui_context),
            platform : Some(platform),
        }
    }
}
