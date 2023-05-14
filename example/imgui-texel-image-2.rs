use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use std::time::Instant;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use wgpu_tutorial_rs::command_buffer::CommandBufferController;
use wgpu_tutorial_rs::gpu::Gpu;
use wgpu_tutorial_rs::imgui_stuff::ImguiController;
use wgpu_tutorial_rs::shader::Shader;
use wgpu_tutorial_rs::texture;

struct App {}

impl App {
    pub fn run() {

        // Set up window and GPU
        let event_loop = EventLoop::new();

        let Gpu {
            window,
            surface,
            device,
            queue,
            hidpi_factor,
            surface_desc,
        } = Gpu::new(&event_loop);

        let mut state = Shader::new(&surface_desc, &device, &queue);

        // Set up dear imgui
        let imgui_controller = ImguiController::new(&window, hidpi_factor);

        let mut imgui = imgui_controller.imgui_context.unwrap();

        let mut platform = imgui_controller.platform.unwrap();

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

        let mut renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

        let texture_context = &mut texture::Context {
            device : &device,
            queue : &queue,
            renderer : &mut renderer,
        };

        let mut last_frame = Instant::now();

        let mut last_cursor = None;

        let mut imgui_region_size : [f32; 2] = [640.0, 480.0];

        // Stores a texture for displaying with imgui::Image(),
        // also as a texture view for rendering into it

        let example_texture_id = texture::Texture::new_texture(texture_context, imgui_region_size);

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

                    let imgui_frame = imgui.frame();

                    let view = main_frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    // Render example normally at background
                    state.update(imgui_frame.io().delta_time);

                    state.setup_camera(&queue, imgui_frame.io().display_size);

                    state.render(&view, &device, &queue);

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

                        // // Resize render target, which is optional
                        // if _size != imgui_region_size && _size[0] >= 1.0 && _size[1] >= 1.0 {
                        //
                        //     imgui_region_size = _size;
                        //
                        //     let scale = &imgui_frame.io().display_framebuffer_scale;
                        //
                        //     let texture_config = TextureConfig {
                        //         size : Extent3d {
                        //             width : (imgui_region_size[0] * scale[0]) as u32,
                        //             height : (imgui_region_size[1] * scale[1]) as u32,
                        //             ..Default::default()
                        //         },
                        //         usage : wgpu::TextureUsages::RENDER_ATTACHMENT
                        //             | wgpu::TextureUsages::TEXTURE_BINDING,
                        //         ..Default::default()
                        //     };
                        //
                        //     renderer_with_imgui.textures.replace(
                        //         example_texture_id,
                        //         Texture::new(&device, &renderer_with_imgui, texture_config),
                        //     );
                        // }

                        // Only render example to example_texture if thw window is not collapsed
                        state.setup_camera(&queue, _size);

                        state.render(
                            renderer.textures.get(example_texture_id).unwrap().view(),
                            &device,
                            &queue,
                        );
                    }

                    let mut command_encoder : wgpu::CommandEncoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

                    if last_cursor != Some(imgui_frame.mouse_cursor()) {

                        last_cursor = Some(imgui_frame.mouse_cursor());

                        platform.prepare_render(imgui_frame, &window);
                    }

                    let mut renderpass =
                        CommandBufferController::create_render_pass(&view, &mut command_encoder);

                    renderer
                        .render(imgui.render(), &queue, &device, &mut renderpass)
                        .expect("Rendering failed");

                    drop(renderpass);

                    queue.submit(Some(command_encoder.finish()));

                    main_frame.present();
                }
                _ => (),
            }

            platform.handle_event(imgui.io_mut(), &window, &event);
        });
    }

    fn main_loop() {

        unimplemented!();
    }
}

fn main() { App::run(); }
