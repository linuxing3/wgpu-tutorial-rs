use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use pollster::block_on;
use std::time::Instant;
use wgpu::Extent3d;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use wgpu_tutorial_rs::app::State;

fn main() {

    env_logger::init();

    // Set up window and GPU
    let event_loop = EventLoop::new();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends : wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let (window, size, surface) = {

        let version = env!("CARGO_PKG_VERSION");

        let window = Window::new(&event_loop).unwrap();

        window.set_inner_size(LogicalSize {
            width : 1280.0,
            height : 720.0,
        });

        window.set_title(&format!("imgui-wgpu {version}"));

        let size = window.inner_size();

        let surface = unsafe {

            instance.create_surface(&window)
        }
        .unwrap();

        (window, size, surface)
    };

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

    // Set up dear imgui
    let mut imgui = imgui::Context::create();

    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);

    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );

    imgui.set_ini_filename(None);

    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

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
        texture_format : surface_desc.format,
        ..Default::default()
    };

    let mut renderer_with_imgui = Renderer::new(&mut imgui, &device, &queue, renderer_config);

    let mut last_frame = Instant::now();

    let mut last_cursor = None;

    let mut imgui_region_size : [f32; 2] = [640.0, 480.0];

    let mut state = State::init(&surface_desc, &device, &queue);

    // Stores a texture for displaying with imgui::Image(),
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

    let texture = Texture::new(&device, &renderer_with_imgui, texture_config);

    let cube_texture_id = renderer_with_imgui.textures.insert(texture);

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

                surface.configure(&device, &surface_desc);
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

                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");

                let imgui_ui = imgui.frame();

                let main_view = main_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Render example normally at background
                state.update(imgui_ui.io().delta_time);

                state.setup_camera(&queue, imgui_ui.io().display_size);

                // NOTE: first render default view with cube background
                state.imgui_render(&main_view, &device, &queue);

                // TODO: use imgui layer

                // Store the new size of Image() or None to indicate that the window is
                // collapsed.
                let mut new_imgui_region_size : Option<[f32; 2]> = None;

                imgui_ui
                    .window("Cube")
                    .size([512.0, 512.0], Condition::FirstUseEver)
                    .build(|| {

                        new_imgui_region_size = Some(imgui_ui.content_region_avail());

                        imgui::Image::new(cube_texture_id, new_imgui_region_size.unwrap())
                            .build(imgui_ui);
                    });

                // NOTE: Draw in imgui window
                if let Some(_size) = new_imgui_region_size {

                    // Resize render target, which is optional
                    if _size != imgui_region_size && _size[0] >= 1.0 && _size[1] >= 1.0 {

                        imgui_region_size = _size;

                        let scale = &imgui_ui.io().display_framebuffer_scale;

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
                            cube_texture_id,
                            Texture::new(&device, &renderer_with_imgui, texture_config),
                        );
                    }

                    // Only render example to example_texture if thw window is not collapsed
                    state.setup_camera(&queue, _size);

                    // NOTE: second render texture
                    state.imgui_render(
                        renderer_with_imgui
                            .textures
                            .get(cube_texture_id)
                            .unwrap()
                            .view(),
                        &device,
                        &queue,
                    );
                }

                let mut main_encoder : wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

                if last_cursor != Some(imgui_ui.mouse_cursor()) {

                    last_cursor = Some(imgui_ui.mouse_cursor());

                    platform.prepare_render(imgui_ui, &window);
                }

                let mut main_rpass = main_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label : None,
                    color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                        view : &main_view,
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
                    .render(imgui.render(), &queue, &device, &mut main_rpass)
                    .expect("Rendering failed");

                drop(main_rpass);

                queue.submit(Some(main_encoder.finish()));

                main_frame.present();
            }
            _ => (),
        }

        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
