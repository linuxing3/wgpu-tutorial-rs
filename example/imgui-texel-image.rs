use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use pollster::block_on;

use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use wgpu_tutorial_rs::texture::Texture as ImguiTexture;
use wgpu_tutorial_rs::{gpu::Gpu, imgui_layer::Layer};
use wgpu_tutorial_rs::{share::create_cube_texels, texture::Context};
use wgpu_tutorial_rs::{share::create_empty_texels, swapchain::Swapchain};

struct State {
    swapchain1 : Swapchain,
    swapchain2 : Swapchain,
}

impl State {
    fn init(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) -> Self {

        // NOTE: texture underlay, data copied

        let texture_texels_1 = create_empty_texels(256, 256);

        let texture_texels_2 = create_cube_texels(128, 128);

        let swapchain1 = Swapchain::new(config, device, queue, 256u32, texture_texels_1);

        let swapchain2 = Swapchain::new(config, device, queue, 123u32, texture_texels_2);

        // Done
        State {
            swapchain1,
            swapchain2,
        }
    }

    pub fn render_swapchain_1<'r>(&'r mut self, rpass : &mut wgpu::RenderPass<'r>) {

        rpass.push_debug_group("Prepare data for draw.");

        rpass.set_pipeline(&self.swapchain1.pipeline);

        rpass.set_bind_group(0, &self.swapchain1.camera_bind_group, &[]);

        rpass.set_bind_group(1, &self.swapchain1.texture_bind_group, &[]);

        rpass.set_index_buffer(
            self.swapchain1.index_buf.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        rpass.set_vertex_buffer(0, self.swapchain1.vertex_buf.slice(..));

        rpass.pop_debug_group();

        rpass.insert_debug_marker("Draw!");

        rpass.draw_indexed(0..self.swapchain1.index_count as u32, 0, 0..1);
    }

    pub fn render_swapchain_2<'r>(&'r mut self, rpass : &mut wgpu::RenderPass<'r>) {

        rpass.push_debug_group("Prepare data for draw.");

        rpass.set_pipeline(&self.swapchain2.pipeline);

        rpass.set_bind_group(0, &self.swapchain2.camera_bind_group, &[]);

        rpass.set_bind_group(1, &self.swapchain2.texture_bind_group, &[]);

        rpass.set_index_buffer(
            self.swapchain2.index_buf.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        rpass.set_vertex_buffer(0, self.swapchain2.vertex_buf.slice(..));

        rpass.pop_debug_group();

        rpass.insert_debug_marker("Draw!");

        rpass.draw_indexed(0..self.swapchain2.index_count as u32, 0, 0..1);
    }

    fn render(&mut self, view : &wgpu::TextureView, device : &wgpu::Device, queue : &wgpu::Queue) {

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

        {

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label : Some("Child Renderpass"),
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

            self.render_swapchain_1(&mut rpass);
        }

        {

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label : Some("Child Renderpass"),
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

            self.render_swapchain_2(&mut rpass);
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

    let mut renderer = Renderer::new(&mut imgui, &gpu.device, &gpu.queue, renderer_config);

    let mut last_frame = Instant::now();

    let mut last_cursor = None;

    let mut imgui_region_size : [f32; 2] = [640.0, 480.0];

    let mut state = State::init(&gpu.surface_desc, &gpu.device, &gpu.queue);

    // NOTE: Stores a imgui texture wrapper for displaying with imgui::Image(),
    // also as a texture view for rendering into it
    let cube_imgui_texture = ImguiTexture::new_texture(
        &mut Context {
            device : &gpu.device,
            queue : &gpu.queue,
            renderer : &mut renderer,
        },
        imgui_region_size,
    );

    let cube_imgui_texture_id = renderer.textures.insert(cube_imgui_texture);

    let empty_imgui_texture = ImguiTexture::new_texture(
        &mut Context {
            device : &gpu.device,
            queue : &gpu.queue,
            renderer : &mut renderer,
        },
        imgui_region_size,
    );

    let empty_imgui_texture_id = renderer.textures.insert(empty_imgui_texture);

    // HACK: imgui layers
    let mut layers : Vec<Layer> = vec![];

    let layer1 = Layer::new(empty_imgui_texture_id, [256.0, 256.0]);

    let layer2 = Layer::new(cube_imgui_texture_id, [256.0, 256.0]);

    layers.push(layer1);

    layers.push(layer2);

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

                if !state.swapchain1.handle_input(event) {

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

                if !state.swapchain2.handle_input(event) {

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
                state.swapchain1.update(ui.io().delta_time);

                state
                    .swapchain1
                    .setup_camera(&gpu.queue, ui.io().display_size);

                // Render example normally at background
                state.swapchain2.update(ui.io().delta_time);

                state
                    .swapchain2
                    .setup_camera(&gpu.queue, ui.io().display_size);

                for layer in &mut layers {

                    let title = "window".to_string() + &layer.id().id().to_string();

                    layer.render(
                        &mut Context {
                            device : &gpu.device,
                            queue : &gpu.queue,
                            renderer : &mut renderer,
                        },
                        ui,
                        &title,
                    );

                    if let Some(new_imgui_region_size) = layer.size() {

                        // Resize render target, which is optional
                        if new_imgui_region_size != imgui_region_size
                            && new_imgui_region_size[0] >= 1.0
                            && new_imgui_region_size[1] >= 1.0
                        {

                            imgui_region_size = new_imgui_region_size;

                            layer.resize(
                                &mut Context {
                                    device : &gpu.device,
                                    queue : &gpu.queue,
                                    renderer : &mut renderer,
                                },
                                ui,
                                imgui_region_size,
                            );
                        }

                        state
                            .swapchain1
                            .setup_camera(&gpu.queue, new_imgui_region_size);

                        state
                            .swapchain2
                            .setup_camera(&gpu.queue, new_imgui_region_size);

                        let view = renderer.textures.get(layer.id()).unwrap().view();

                        // NOTE: use a separate renderpass
                        state.render(view, &gpu.device, &gpu.queue);
                    }
                }

                if last_cursor != Some(ui.mouse_cursor()) {

                    last_cursor = Some(ui.mouse_cursor());

                    platform.prepare_render(ui, &window);
                }

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
                        label : Some("Main renderpass"),
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

                // NOTE: render imgui raw data
                renderer
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
