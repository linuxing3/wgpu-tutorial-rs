use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use imgui_winit_support::WinitPlatform;
use pollster::block_on;

use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use wgpu_tutorial_rs::swapchain::Swapchain;
use wgpu_tutorial_rs::texture::Context;
use wgpu_tutorial_rs::{gpu::Gpu, imgui_layer::Layer};

struct State {
    swapchain : Swapchain,
}

impl State {
    fn init(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) -> Self {

        let swapchain = Swapchain::new(config, device, queue);

        // Done
        State { swapchain }
    }

    pub fn render_with_rpass<'r>(&'r mut self, rpass : &mut wgpu::RenderPass<'r>) {

        rpass.push_debug_group("Prepare data for draw.");

        rpass.set_pipeline(&self.swapchain.pipeline);

        rpass.set_bind_group(0, &self.swapchain.bind_group, &[]);

        rpass.set_index_buffer(
            self.swapchain.index_buf.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        rpass.set_vertex_buffer(0, self.swapchain.vertex_buf.slice(..));

        rpass.pop_debug_group();

        rpass.insert_debug_marker("Draw!");

        rpass.draw_indexed(0..self.swapchain.index_count as u32, 0, 0..1);
    }

    fn render(&mut self, view : &wgpu::TextureView, device : &wgpu::Device, queue : &wgpu::Queue) {

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

        {

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label : None,
                color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target : None,
                    ops : wgpu::Operations {
                        load : wgpu::LoadOp::Clear(wgpu::Color {
                            r : 0.1,
                            g : 0.2,
                            b : 0.3,
                            a : 1.0,
                        }),
                        store : true,
                    },
                })],
                depth_stencil_attachment : None,
            });

            self.render_with_rpass(&mut rpass);
        }

        queue.submit(Some(encoder.finish()));
    }
}

async fn run() {

    env_logger::init();

    // Set up window and GPU
    let event_loop = EventLoop::new();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends : wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let (mut window, surface) = {

        let version = env!("CARGO_PKG_VERSION");

        let window = Window::new(&event_loop).unwrap();

        window.set_inner_size(LogicalSize {
            width : 1280.0,
            height : 720.0,
        });

        window.set_title(&format!("imgui-wgpu {version}"));

        let surface = unsafe {

            instance.create_surface(&window)
        }
        .unwrap();

        (window, surface)
    };

    let gpu = Gpu::new(&mut window, &instance, &surface);

    // NOTE: Set up dear imgui
    let mut imgui = imgui::Context::create();

    let mut platform = WinitPlatform::init(&mut imgui);

    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );

    imgui.set_ini_filename(None);

    let font_size = (13.0 * gpu.hidpi_factor) as f32;

    imgui.io_mut().font_global_scale = (1.0 / gpu.hidpi_factor) as f32;

    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config : Some(imgui::FontConfig {
            oversample_h : 1,
            pixel_snap_h : true,
            size_pixels : font_size,
            ..Default::default()
        }),
    }]);

    let renderer_config = RendererConfig {
        texture_format : gpu.surface_desc.format,
        ..Default::default()
    };

    let mut renderer_with_imgui =
        Renderer::new(&mut imgui, &gpu.device, &gpu.queue, renderer_config);

    let mut last_frame = Instant::now();

    let mut last_cursor = None;

    let mut imgui_region_size : [f32; 2] = [640.0, 480.0];

    let mut state = State::init(&gpu.surface_desc, &gpu.device, &gpu.queue);

    // NOTE: Stores a texture for displaying with imgui::Image(),
    // also as a texture view for rendering into it

    let texture_config = TextureConfig {
        size : wgpu::Extent3d {
            width : imgui_region_size[0] as u32,
            height : imgui_region_size[1] as u32,
            ..Default::default()
        },
        usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        ..Default::default()
    };

    let texture = Texture::new(&gpu.device, &renderer_with_imgui, texture_config);

    let example_texture_id = renderer_with_imgui.textures.insert(texture);

    // HACK: imgui layers
    let mut layers : Vec<Layer> = vec![];

    let x_layer = Layer::new(example_texture_id, [256.0, 256.0]);

    layers.push(x_layer);

    // Event loop
    event_loop.run(move |event, _, control_flow| {

        *control_flow = if cfg!(feature = "metal-auto-capture") {

            ControlFlow::Exit
        } else {

            ControlFlow::Poll
        };

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {

                // TIP: will raise stencil error if not resize
                let size = window.inner_size();

                let surface_desc = wgpu::SurfaceConfiguration {
                    usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format : wgpu::TextureFormat::Bgra8UnormSrgb,
                    width : size.width,
                    height : size.height,
                    present_mode : wgpu::PresentMode::Fifo,
                    alpha_mode : wgpu::CompositeAlphaMode::Auto,
                    view_formats : vec![wgpu::TextureFormat::Bgra8Unorm],
                };

                surface.configure(&gpu.device, &surface_desc);
            }
            Event::WindowEvent {
                ref event,
                window_id: _,
            } => {
                if !state.swapchain.handle_input(event) {

                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawEventsCleared => {

                // --------------------------------------------------------------------
                // HACK: imgui io - imgui-winit-support platform - winit window
                let now = Instant::now();

                imgui.io_mut().update_delta_time(now - last_frame);

                last_frame = now;

                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");

                let ui = imgui.frame();

                // Render example normally at background
                state.swapchain.update(ui.io().delta_time);

                state
                    .swapchain
                    .setup_camera(&gpu.queue, ui.io().display_size);

                for layer in &mut layers {

                    let texture_context = &mut Context {
                        device : &gpu.device,
                        queue : &gpu.queue,
                        renderer : &mut renderer_with_imgui,
                    };

                    layer.render(texture_context, ui);

                    if let Some(new_imgui_region_size) = layer.size() {

                        // Resize render target, which is optional
                        if new_imgui_region_size != imgui_region_size
                            && new_imgui_region_size[0] >= 1.0
                            && new_imgui_region_size[1] >= 1.0
                        {

                            imgui_region_size = new_imgui_region_size;

                            layer.resize(texture_context, ui, imgui_region_size);
                        }

                        state
                            .swapchain
                            .setup_camera(&gpu.queue, new_imgui_region_size);

                        let view = renderer_with_imgui
                            .textures
                            .get(example_texture_id)
                            .unwrap()
                            .view();

                        state.render(view, &gpu.device, &gpu.queue);
                    }
                }

                if last_cursor != Some(ui.mouse_cursor()) {

                    last_cursor = Some(ui.mouse_cursor());

                    platform.prepare_render(ui, &window);
                }

                // --------------------------------------------------------------------
                // NOTE: render all
                let main_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {

                        eprintln!("dropped frame: {e:?}");

                        return;
                    }
                };

                let view = main_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut command_encoder : wgpu::CommandEncoder = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

                let mut renderpass =
                    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label : None,
                        color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                            view : &view,
                            resolve_target : None,
                            ops : wgpu::Operations {
                                load : wgpu::LoadOp::Load, // Do not clear
                                // load: wgpu::LoadOp::Clear(clear_color),
                                store : true,
                            },
                        })],
                        depth_stencil_attachment : None,
                    });

                renderer_with_imgui
                    .render(imgui.render(), &gpu.queue, &gpu.device, &mut renderpass)
                    .expect("Rendering failed");

                drop(renderpass);

                // NOTE: render background image
                state.render(&view, &gpu.device, &gpu.queue);

                gpu.queue.submit(Some(command_encoder.finish()));

                main_frame.present();
            }
            _ => (),
        }

        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}

fn main() { block_on(run()); }
