use bytemuck::{Pod, Zeroable};
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use pollster::block_on;
use std::time::Instant;
use wgpu::{include_wgsl, util::DeviceExt, Extent3d};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const OPENGL_TO_WGPU_MATRIX : cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

// Example code modified from https://github.com/gfx-rs/wgpu-rs/tree/master/examples/cube
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]

struct Vertex {
    _pos : [f32; 4],
    _tex_coord : [f32; 2],
}

fn vertex(pos : [i8; 3], tc : [i8; 2]) -> Vertex {

    Vertex {
        _pos : [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord : [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {

    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];

    let index_data : &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_texels(size : usize) -> Vec<u8> {

    (0..size * size)
        .map(|id| {

            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;

            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;

            let (mut x, mut y, mut count) = (cx, cy, 0);

            while count < 0xFF && x * x + y * y < 4.0 {

                let old_x = x;

                x = x * x - y * y + cx;

                y = 2.0 * old_x * y + cy;

                count += 1;
            }

            count
        })
        .collect()
}

struct State {
    vertex_buf : wgpu::Buffer,
    index_buf : wgpu::Buffer,
    index_count : usize,
    bind_group : wgpu::BindGroup,
    uniform_buf : wgpu::Buffer,
    pipeline : wgpu::RenderPipeline,
    time : f32,
}

impl State {
    fn generate_matrix(aspect_ratio : f32) -> cgmath::Matrix4<f32> {

        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);

        let mx_view = cgmath::Matrix4::look_at_rh(
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );

        let mx_correction = OPENGL_TO_WGPU_MATRIX;

        mx_correction * mx_projection * mx_view
    }
}

impl State {
    fn init(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) -> Self {

        use std::mem;

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<Vertex>();

        let (vertex_data, index_data) = create_vertices();

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Vertex Buffer"),
            contents : bytemuck::cast_slice(&vertex_data),
            usage : wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Index Buffer"),
            contents : bytemuck::cast_slice(&index_data),
            usage : wgpu::BufferUsages::INDEX,
        });

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label : None,
            entries : &[
                wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::VERTEX,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset : false,
                        min_binding_size : wgpu::BufferSize::new(64),
                    },
                    count : None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 1,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Texture {
                        multisampled : false,
                        sample_type : wgpu::TextureSampleType::Uint,
                        view_dimension : wgpu::TextureViewDimension::D2,
                    },
                    count : None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : None,
            bind_group_layouts : &[&bind_group_layout],
            push_constant_ranges : &[],
        });

        // Create the texture
        let texture_size = 256u32;

        let texture_texels = create_texels(texture_size as usize);

        let cube_texture_extent = wgpu::Extent3d {
            width : texture_size,
            height : texture_size,
            depth_or_array_layers : 1,
        };

        let cube_texture = device.create_texture(&wgpu::TextureDescriptor {
            label : None,
            size : cube_texture_extent,
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format : wgpu::TextureFormat::R8Uint,
            usage : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats : &[wgpu::TextureFormat::R8Uint],
        });

        let cube_texture_view = cube_texture.create_view(&wgpu::TextureViewDescriptor::default());

        queue.write_texture(
            cube_texture.as_image_copy(),
            &texture_texels,
            wgpu::ImageDataLayout {
                offset : 0,
                bytes_per_row : Some(std::num::NonZeroU32::new(texture_size).unwrap()),
                rows_per_image : None,
            },
            cube_texture_extent,
        );

        // Create other resources
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);

        let mx_ref : &[f32; 16] = mx_total.as_ref();

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Uniform Buffer"),
            contents : bytemuck::cast_slice(mx_ref),
            usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &bind_group_layout,
            entries : &[
                wgpu::BindGroupEntry {
                    binding : 0,
                    resource : uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 1,
                    resource : wgpu::BindingResource::TextureView(&cube_texture_view),
                },
            ],
            label : None,
        });

        let shader = device.create_shader_module(include_wgsl!("../assets/shaders/cube.wgsl"));

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride : vertex_size as wgpu::BufferAddress,
            step_mode : wgpu::VertexStepMode::Vertex,
            attributes : &[
                wgpu::VertexAttribute {
                    format : wgpu::VertexFormat::Float32x4,
                    offset : 0,
                    shader_location : 0,
                },
                wgpu::VertexAttribute {
                    format : wgpu::VertexFormat::Float32x2,
                    offset : 4 * 4,
                    shader_location : 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label : None,
            layout : Some(&pipeline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers : &vertex_buffers,
            },
            fragment : Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                targets : &[Some(config.format.into())],
            }),
            primitive : wgpu::PrimitiveState {
                cull_mode : Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil : None,
            multisample : wgpu::MultisampleState::default(),
            multiview : None,
        });

        // Done
        State {
            vertex_buf,
            index_buf,
            index_count : index_data.len(),
            bind_group,
            uniform_buf,
            pipeline,
            time : 0.0,
        }
    }

    fn update(&mut self, delta_time : f32) { self.time += delta_time; }

    fn setup_camera(&mut self, queue : &wgpu::Queue, size : [f32; 2]) {

        let mx_total = Self::generate_matrix(size[0] / size[1]);

        let mx_ref : &[f32; 16] = mx_total.as_ref();

        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(mx_ref));
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

            rpass.set_pipeline(&self.pipeline);

            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);

            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));

            rpass.pop_debug_group();

            rpass.insert_debug_marker("Draw!");

            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
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
                            Texture::new(&device, &renderer_with_imgui, texture_config),
                        );
                    }

                    // Only render example to example_texture if thw window is not collapsed
                    state.setup_camera(&queue, _size);

                    state.render(
                        renderer_with_imgui
                            .textures
                            .get(example_texture_id)
                            .unwrap()
                            .view(),
                        &device,
                        &queue,
                    );
                }

                let mut command_encoder : wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

                if last_cursor != Some(imgui_frame.mouse_cursor()) {

                    last_cursor = Some(imgui_frame.mouse_cursor());

                    platform.prepare_render(imgui_frame, &window);
                }

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
