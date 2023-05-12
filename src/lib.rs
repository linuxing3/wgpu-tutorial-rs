// lib.rs

mod layer;
mod layer_renderer;
mod texture;

// use crate::layer;
// use crate::layer_renderer;
// use crate::texture;

extern crate imgui_winit_support;

use __core::borrow::BorrowMut;
use layer::Layer;
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

struct VertexBasic {
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

// NOTE: https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/#a-controller-for-our-camera

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    eye : cgmath::Point3<f32>,
    target : cgmath::Point3<f32>,
    up : cgmath::Vector3<f32>,
    aspect : f32,
    fovy : f32,
    znear : f32,
    zfar : f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {

        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

pub struct CameraUniform {
    view_proj : [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {

        use cgmath::SquareMatrix;

        Self {
            view_proj : cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera : &Camera) {

        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

pub struct CameraController {
    speed : f32,
    is_up_pressed : bool,
    is_down_pressed : bool,
    is_forward_pressed : bool,
    is_backward_pressed : bool,
    is_left_pressed : bool,
    is_right_pressed : bool,
}

impl CameraController {
    pub fn new(speed : f32) -> Self {

        Self {
            speed,
            is_up_pressed : false,
            is_down_pressed : false,
            is_forward_pressed : false,
            is_backward_pressed : false,
            is_left_pressed : false,
            is_right_pressed : false,
        }
    }

    pub fn process_events(&mut self, event : &WindowEvent) -> bool {

        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {

                let is_pressed = *state == ElementState::Pressed;

                match keycode {
                    VirtualKeyCode::Space => {

                        self.is_up_pressed = is_pressed;

                        true
                    }
                    VirtualKeyCode::LShift => {

                        self.is_down_pressed = is_pressed;

                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {

                        self.is_forward_pressed = is_pressed;

                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {

                        self.is_left_pressed = is_pressed;

                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {

                        self.is_backward_pressed = is_pressed;

                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {

                        self.is_right_pressed = is_pressed;

                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera : &mut Camera) {

        use cgmath::InnerSpace;

        let forward = camera.target - camera.eye;

        let forward_norm = forward.normalize();

        let forward_mag = forward.magnitude();

        // NOTE: z ++ --
        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {

            camera.eye += forward_norm * self.speed;
        }

        if self.is_backward_pressed {

            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;

        let forward_mag = forward.magnitude();

        if self.is_right_pressed {

            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }

        if self.is_left_pressed {

            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}

struct State {
    surface : wgpu::Surface,
    device : wgpu::Device,
    queue : wgpu::Queue,
    config : wgpu::SurfaceConfiguration,
    size : winit::dpi::PhysicalSize<u32>,
    window : Window,

    // Pipeline
    render_pipeline : wgpu::RenderPipeline,
    vertex_buffer : wgpu::Buffer,
    index_buffer : wgpu::Buffer,
    // num_vertices : u32,
    num_indices : u32,

    // texture
    diffuse_bind_group : wgpu::BindGroup,
    diffuse_texture : texture::Texture,

    // camera
    camera : Camera,
    camera_controller : CameraController,
    camera_uniform : CameraUniform,
    camera_buffer : wgpu::Buffer,
    camera_bind_group : wgpu::BindGroup,

    clear_color : wgpu::Color,

    // imgui
    imgui_context : imgui::Context,
    last_frame : Instant,
    last_cursor : Option<imgui::MouseCursor>,
    renderer : Renderer,
    platform : imgui_winit_support::WinitPlatform,
    demo_open : bool,

    // layers
    layers : Vec<Layer>,
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

        //
        // Surface
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

        // Device and queue with features

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

        // surfaces formats

        let surface_caps = surface.get_capabilities(&adapter);

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

        // NOTE: images
        // [doc] https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#loading-an-image-from-a-file

        // NOTE: camera

        let camera = Camera {
            eye : (0.0, 1.0, 2.0).into(),
            target : (0.0, 0.0, 0.0).into(),
            up : cgmath::Vector3::unit_y(),
            aspect : config.width as f32 / config.height as f32,
            fovy : 45.0,
            znear : 0.1,
            zfar : 100.0,
        };

        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();

        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Camera Buffer"),
            contents : bytemuck::cast_slice(&[camera_uniform]),
            usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries : &[wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::VERTEX,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset : false,
                        min_binding_size : None,
                    },
                    count : None,
                }],
                label : Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &camera_bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : camera_buffer.as_entire_binding(),
            }],
            label : Some("camera_bind_group"),
        });

        // NOTE: Imgui

        // [doc] file:///home/vagrant/workspace/rust/wgpu-tutorial-rs/target/doc/imgui_winit_support/index.html
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

        // NOTE: Set up dear imgui wgpu renderer
        //
        let renderer_config = RendererConfig {
            texture_format : config.format,
            ..Default::default()
        };

        let mut renderer = Renderer::new(&mut imgui_context, &device, &queue, renderer_config);

        let last_frame = Instant::now();

        // NOTE: setup imgui layers, holds widgets

        let diffuse_bytes = include_bytes!("../assets/images/happy-tree.png");

        let mut layer_context = texture::Context {
            device : &device,
            queue : &queue,
            renderer : &mut renderer,
        };

        let diffuse_texture =
            texture::Texture::from_bytes(diffuse_bytes, &layer_context, "happy-tree.png").unwrap(); // CHANGED!

        let difusse_texture_entries = [
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
        ];

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries : &difusse_texture_entries,
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

        // NOTE: Normal triangle render stuff

        // render_pipeline
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../assets/shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout], // NOTE:
                push_constant_ranges: &[],
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

        // NOTE: vertex buffer

        let vertex_buffer_desc = &wgpu::util::BufferInitDescriptor {
            label : Some("Vertex Buffer"),
            contents : bytemuck::cast_slice(VERTICES),
            usage : wgpu::BufferUsages::VERTEX,
        };

        let vertex_buffer = device.create_buffer_init(vertex_buffer_desc);

        // NOTE: index buffer
        let index_buffer_desc = &wgpu::util::BufferInitDescriptor {
            label : Some("Index Buffer"),
            contents : bytemuck::cast_slice(INDICES),
            usage : wgpu::BufferUsages::INDEX,
        };

        let index_buffer = device.create_buffer_init(index_buffer_desc);

        let num_indices = INDICES.len() as u32;

        // ---------------------------------------------------------------------------------
        // NOTE: prepare imgui layers

        let mut layers = vec![];

        let happy_bytes = include_bytes!("../assets/images/happy-tree.png");

        let x_layer = layer::Layer::new(&mut layer_context, happy_bytes);

        layers.push(x_layer);

        // ---------------------------------------------------------------------------------

        // NOTE: for window event detection, updating imgui state, etc

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
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            renderer,
            imgui_context,
            platform,
            last_frame,
            demo_open : true,
            last_cursor,
            layers,
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

    fn input(&mut self, event : &WindowEvent) -> bool {

        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {

        self.camera_controller.update_camera(&mut self.camera);

        self.camera_uniform.update_view_proj(&self.camera);

        // NOTE: uniform information -> uniform buffer
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        // NOTE: imgui timer
        let delta_s = self.last_frame.elapsed();

        let now = Instant::now();

        let io = self.imgui_context.io_mut();

        io.update_delta_time(delta_s);

        self.last_frame = now;

        // NOTE: imgui ui = frame -> layers -> widgets

        let main_frame = self.surface.get_current_texture()?;

        self.platform
            .prepare_frame(io, &self.window)
            .expect("Failed to prepare frame");

        let ui = self.imgui_context.frame();

        // NOTE: prepare imgui layers

        let width = self.window.inner_size().width;

        let height = self.window.inner_size().height;

        let context = &mut texture::Context {
            device : &self.device,
            queue : &self.queue,
            renderer : &mut self.renderer,
        };

        for layer in &mut self.layers {

            if let Some(window) = ui
                .window("Gallery")
                .size(
                    [width as f32, height as f32],
                    imgui::Condition::FirstUseEver,
                )
                .begin()
            {

                layer.render(context, &ui);

                window.end();
            };
        }

        // NOTE: prepare render
        if self.last_cursor != ui.mouse_cursor() {

            self.last_cursor = ui.mouse_cursor();

            self.platform.prepare_render(ui, &self.window);
        }

        let main_view = main_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label : Some("Render Encoder"),
                });

        // Render pass scope
        {

            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label : Some("Render Pass"),
                color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                    view : &main_view,
                    resolve_target : None,
                    ops : wgpu::Operations {
                        load : wgpu::LoadOp::Clear(self.clear_color),
                        store : true,
                    },
                })],
                depth_stencil_attachment : None,
            });

            // NOTE: bindings
            // pipeline <-> buffers...
            // texture/sampler -> fragment shader
            // camera -> uniform buffer -> vertex/index shader
            // vertex -> vertex buffer -> vertex shader
            // index -> index buffer -> vertex shader

            render_pass.set_pipeline(&self.render_pipeline); // 2.

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NOTE: texture with pipeline

            render_pass.set_bind_group(1, &self.camera_bind_group, &[]); // NOTE: camera with 3D effect

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..)); //NOTE: vertex cached with uniform 3d effect

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // NOTE: index cached

            // NOTE: draw shapes from pipeline on surface

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 3.NOTE: more parameter than draw method

            // NOTE: render imgui

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

        // NOTE: submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(command_encoder.finish()));

        main_frame.present();

        Ok(())
    }
}

// run entry
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

                // render entry
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
