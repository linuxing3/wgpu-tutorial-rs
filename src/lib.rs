// lib.rs

mod texture;

extern crate imgui_winit_support;

use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};

use wgpu::util::DeviceExt;

struct State {
    surface : wgpu::Surface,
    device : wgpu::Device,
    queue : wgpu::Queue,
    config : wgpu::SurfaceConfiguration,
    size : winit::dpi::PhysicalSize<u32>,
    window : Window,
    render_pipeline : wgpu::RenderPipeline,
    vertex_buffer : wgpu::Buffer,
    index_buffer : wgpu::Buffer,
    // num_vertices : u32,
    num_indices : u32,

    diffuse_bind_group : wgpu::BindGroup,
    diffuse_texture : texture::Texture, // NEW

    clear_color : wgpu::Color,

    // imgui
    imgui_context : imgui::Context,
    last_frame : Instant,
    last_cursor : Option<imgui::MouseCursor>,
    renderer : Renderer,
    platform : imgui_winit_support::WinitPlatform,
    demo_open : bool,
}

impl State {
    async fn new(window : Window) -> Self {

        let hidpi_factor = window.scale_factor();

        let clear_color = wgpu::Color {
            r : 0.1,
            g : 0.2,
            b : 0.3,
            a : 1.0,
        };

        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends : wgpu::Backends::all(),
            dx12_shader_compiler : Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe {

            instance.create_surface(&window)
        }
        .unwrap();

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .filter(|adapter| {

                // Check if this adapter supports our surface
                adapter.is_surface_supported(&surface)
            })
            .next()
            .unwrap();

        // device and queue with features

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features : wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits : if cfg!(target_arch = "wasm32") {

                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {

                        wgpu::Limits::default()
                    },
                    label : None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        // surfaces

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a
        // different one will result all the colors coming out darker. If you
        // want to support non sRGB surfaces, you'll need to account for that
        // when drawing to the frame.
        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     // .find(|p| p.describe().srgb = true)
        //     .copied()
        //     .next()
        //     .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
            format : wgpu::TextureFormat::Bgra8UnormSrgb,
            width : size.width,
            height : size.height,
            present_mode : surface_caps.present_modes[0],
            alpha_mode : surface_caps.alpha_modes[0],
            view_formats : vec![wgpu::TextureFormat::Bgra8Unorm],
        };

        //
        surface.configure(&device, &config);

        // NOTE:
        // [doc] https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#loading-an-image-from-a-file

        let diffuse_bytes = include_bytes!("happy-tree.png");

        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap(); // CHANGED!

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries : &[
                    wgpu::BindGroupLayoutEntry {
                        binding : 0,
                        visibility : wgpu::ShaderStages::FRAGMENT,
                        ty : wgpu::BindingType::Texture {
                            multisampled : false,
                            view_dimension : wgpu::TextureViewDimension::D2,
                            sample_type : wgpu::TextureSampleType::Float { filterable : true },
                        },
                        count : None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding : 1,
                        visibility : wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count : None,
                    },
                ],
                label : Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &texture_bind_group_layout,
            entries : &[
                wgpu::BindGroupEntry {
                    binding : 0,
                    resource : wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding : 1,
                    resource : wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label : Some("diffuse_bind_group"),
        });

        // NOTE:
        // [doc] file:///home/vagrant/workspace/rust/wgpu-tutorial-rs/target/doc/imgui_winit_support/index.html
        // Set up dear imgui
        let mut imgui_context = imgui::Context::create();

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

        platform.attach_window(
            imgui_context.io_mut(),
            &window,
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

        //
        // Set up dear imgui wgpu renderer
        //
        let renderer_config = RendererConfig {
            texture_format : config.format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui_context, &device, &queue, renderer_config);

        let last_frame = Instant::now();

        // NOTE: normal triangle render
        // render_pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label : Some("Render Pipeline Layout"),
                bind_group_layouts : &[&texture_bind_group_layout],
                push_constant_ranges : &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label : Some("Render Pipeline"),
            layout : Some(&render_pipeline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",     // 1.
                buffers : &[Vertex::desc()], // 2.
            },
            fragment : Some(wgpu::FragmentState {
                // 3.
                module : &shader,
                entry_point : "fs_main",
                targets : &[Some(wgpu::ColorTargetState {
                    // 4.
                    format : wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend : Some(wgpu::BlendState::REPLACE),
                    write_mask : wgpu::ColorWrites::ALL,
                })],
            }),
            primitive : wgpu::PrimitiveState {
                topology : wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format : None,
                front_face : wgpu::FrontFace::Ccw, // 2.
                cull_mode : Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode : wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth : false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative : false,
            },
            depth_stencil : None, // 1.
            multisample : wgpu::MultisampleState {
                count : 1,                         // 2.
                mask : !0,                         // 3.
                alpha_to_coverage_enabled : false, // 4.
            },
            multiview : None, // 5.
        });

        // vertex buffer
        let vertex_buffer_desc = &wgpu::util::BufferInitDescriptor {
            label : Some("Vertex Buffer"),
            contents : bytemuck::cast_slice(VERTICES),
            usage : wgpu::BufferUsages::VERTEX,
        };

        let vertex_buffer = device.create_buffer_init(vertex_buffer_desc);

        // index buffer
        let index_buffer_desc = &wgpu::util::BufferInitDescriptor {
            label : Some("Index Buffer"),
            contents : bytemuck::cast_slice(INDICES),
            usage : wgpu::BufferUsages::INDEX,
        };

        let index_buffer = device.create_buffer_init(index_buffer_desc);

        let num_indices = INDICES.len() as u32;

        let last_cursor = None;

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            renderer,
            imgui_context,
            platform,
            last_frame,
            demo_open : true,
            last_cursor,
        }
    }

    pub fn window(&self) -> &Window { &self.window }

    // impl State
    pub fn resize(&mut self, new_size : winit::dpi::PhysicalSize<u32>) {

        if new_size.width > 0 && new_size.height > 0 {

            self.size = new_size;

            self.config = wgpu::SurfaceConfiguration {
                usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
                format : wgpu::TextureFormat::Bgra8UnormSrgb,
                width : new_size.width,
                height : new_size.height,
                present_mode : wgpu::PresentMode::Fifo,
                alpha_mode : wgpu::CompositeAlphaMode::Auto,
                view_formats : vec![wgpu::TextureFormat::Bgra8Unorm],
            };

            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, _event : &WindowEvent) -> bool { false }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        // imgui stuff
        let delta_s = self.last_frame.elapsed();

        let now = Instant::now();

        self.imgui_context
            .io_mut()
            .update_delta_time(now - self.last_frame);

        self.last_frame = now;

        let frame = self.surface.get_current_texture()?;

        self.platform
            .prepare_frame(self.imgui_context.io_mut(), &self.window)
            .expect("Failed to prepare frame");

        let ui = self.imgui_context.frame();

        {

            let window = ui.window("Hello world");

            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(|| {

                    ui.text("Hello world!");

                    ui.text("This...is...imgui-rs on WGPU!");

                    ui.separator();

                    let mouse_pos = ui.io().mouse_pos;

                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            let window = ui.window("Hello too");

            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(|| {

                    ui.text(format!("Frametime: {delta_s:?}"));
                });

            ui.show_demo_window(&mut self.demo_open);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label : Some("Render Encoder"),
            });

        if self.last_cursor != ui.mouse_cursor() {

            self.last_cursor = ui.mouse_cursor();

            self.platform.prepare_render(ui, &self.window);
        }

        // Render triangle
        {

            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label : Some("Render Pass"),
                color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                    view : &view,
                    resolve_target : None,
                    ops : wgpu::Operations {
                        load : wgpu::LoadOp::Clear(self.clear_color),
                        store : true,
                    },
                })],
                depth_stencil_attachment : None,
            });

            render_pass.set_pipeline(&self.render_pipeline); // 2.

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 3.

            // NOTE:
            // render imgui

            self.renderer
                .render(
                    self.imgui_context.render(),
                    &self.queue,
                    &self.device,
                    &mut render_pass,
                )
                .expect("Render imgui failed");

            drop(render_pass);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        frame.present();

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

struct Vertex_Basic {
    position : [f32; 3],
    color : [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

struct Vertex {
    position : [f32; 3],
    tex_coords : [f32; 2], // NEW!
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {

        const ATTRIBS : [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride : std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode : wgpu::VertexStepMode::Vertex,
            attributes : &ATTRIBS,
            // attributes : &[
            //     wgpu::VertexAttribute {
            //         offset : 0,
            //         shader_location : 0,
            //         format : wgpu::VertexFormat::Float32x3,
            //     },
            //     wgpu::VertexAttribute {
            //         offset : std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            //         shader_location : 1,
            //         format : wgpu::VertexFormat::Float32x3,
            //     },
            // ],
        }
    }
}

const VERTICES : &[Vertex] = &[
    // Changed
    Vertex {
        position : [-0.0868241, 0.49240386, 0.0],
        tex_coords : [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position : [-0.49513406, 0.06958647, 0.0],
        tex_coords : [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position : [-0.21918549, -0.44939706, 0.0],
        tex_coords : [0.28081453, 0.949397],
    }, // C
    Vertex {
        position : [0.35966998, -0.3473291, 0.0],
        tex_coords : [0.85967, 0.84732914],
    }, // D
    Vertex {
        position : [0.44147372, 0.2347359, 0.0],
        tex_coords : [0.9414737, 0.2652641],
    }, // E
];

const INDICES : &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

// const VERTICES : &[Vertex] = &[
//     Vertex {
//         position : [0.0, 0.5, 0.0],
//         color : [1.0, 0.0, 0.0],
//     },
//     Vertex {
//         position : [-0.5, -0.5, 0.0],
//         color : [0.0, 1.0, 0.0],
//     },
//     Vertex {
//         position : [0.5, -0.5, 0.0],
//         color : [0.0, 0.0, 1.0],
//     },
// ];

pub async fn run() {

    env_logger::init();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_inner_size(LogicalSize {
        width : 1280.0,
        height : 720.0,
    });

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {

        match event {
            Event::RedrawEventsCleared => {

                state.update();

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {

                state.update();
            }
            Event::MainEventsCleared => {

                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            //
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {

                state.resize(state.window().inner_size());
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {

                if !state.input(event) {

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
                        WindowEvent::Resized(physical_size) => {

                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {

                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        state
            .platform
            .handle_event(state.imgui_context.io_mut(), &state.window, &event);
    });
}
