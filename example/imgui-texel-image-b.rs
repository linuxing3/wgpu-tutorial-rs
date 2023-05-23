use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use imgui_winit_support::WinitPlatform;

use std::time::Instant;
use wgpu::Extent3d;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use wgpu_tutorial_rs::gpu::Gpu;
use wgpu_tutorial_rs::swapchain::Swapchain;

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

        queue.submit(Some(encoder.finish()));
    }
}

fn main() {

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

    //
    // Set up dear imgui wgpu renderer
    //
    // let clear_color = wgpu::Color {
    //     r: 0.1,
    //     g: 0.2,
    //     b: 0.3,
    //     a: 1.0,
    // };

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
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {

                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawEventsCleared => {

                let now = Instant::now();

                imgui.io_mut().update_delta_time(now - last_frame);

                last_frame = now;

                let main_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {

                        eprintln!("dropped frame: {e:?}");

                        return;
                    }
                };

                // TIP: imgui io - imgui-winit-support platform - winit window
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");

                let imgui_frame = imgui.frame();

                let view = main_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Render example normally at background
                state.swapchain.update(imgui_frame.io().delta_time);

                state
                    .swapchain
                    .setup_camera(&gpu.queue, imgui_frame.io().display_size);

                state.render(&view, &gpu.device, &gpu.queue);

                // Store the new size of Image() or None to indicate that the window is
                // collapsed.
                let mut new_imgui_region_size : Option<[f32; 2]> = None;

                imgui_frame
                    .window("Cube")
                    .size([512.0, 512.0], Condition::FirstUseEver)
                    .build(|| {

                        new_imgui_region_size = Some(imgui_frame.content_region_avail());

                        imgui::Image::new(example_texture_id, new_imgui_region_size.unwrap())
                            .build(imgui_frame);
                    });

                if let Some(_size) = new_imgui_region_size {

                    // Resize render target, which is optional
                    if _size != imgui_region_size && _size[0] >= 1.0 && _size[1] >= 1.0 {

                        imgui_region_size = _size;

                        let scale = &imgui_frame.io().display_framebuffer_scale;

                        let texture_config = TextureConfig {
                            size : Extent3d {
                                width : (imgui_region_size[0] * scale[0]) as u32,
                                height : (imgui_region_size[1] * scale[1]) as u32,
                                ..Default::default()
                            },
                            usage : wgpu::TextureUsages::RENDER_ATTACHMENT
                                | wgpu::TextureUsages::TEXTURE_BINDING,
                            ..Default::default()
                        };

                        renderer_with_imgui.textures.replace(
                            example_texture_id,
                            Texture::new(&gpu.device, &renderer_with_imgui, texture_config),
                        );
                    }

                    // Only render example to example_texture if thw window is not collapsed
                    state.swapchain.setup_camera(&gpu.queue, _size);

                    state.render(
                        renderer_with_imgui
                            .textures
                            .get(example_texture_id)
                            .unwrap()
                            .view(),
                        &gpu.device,
                        &gpu.queue,
                    );
                }

                let mut command_encoder : wgpu::CommandEncoder = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

                if last_cursor != Some(imgui_frame.mouse_cursor()) {

                    last_cursor = Some(imgui_frame.mouse_cursor());

                    platform.prepare_render(imgui_frame, &window);
                }

                // NOTE: render
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

                gpu.queue.submit(Some(command_encoder.finish()));

                main_frame.present();
            }
            _ => (),
        }

        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
