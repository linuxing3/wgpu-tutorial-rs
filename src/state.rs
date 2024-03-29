extern crate imgui_winit_support;

use cgmath::prelude::*;
use wgpu::BindingResource::TextureView;
use wgpu::BufferUsages;

use crate::camera::*;
use crate::imgui_layer::Layer;
use crate::model::Model;
use crate::resource;
use crate::share::*;
use crate::texture;
use wgpu::util::DeviceExt;

use std::time::Instant;
use winit::{event::WindowEvent, window::Window};

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};

pub struct State {
    pub surface : wgpu::Surface,
    pub device : wgpu::Device,
    pub queue : wgpu::Queue,
    pub config : wgpu::SurfaceConfiguration,
    pub size : winit::dpi::PhysicalSize<u32>,
    pub window : Window,

    // Pipeline
    pub render_pipeline : wgpu::RenderPipeline,

    // texture
    pub diffuse_bind_group : wgpu::BindGroup,
    pub diffuse_texture : texture::Texture,
    pub depth_texture : texture::Texture,
    pub obj_model : Model,

    // instance
    instances : Vec<Instance>,
    instance_buffer : wgpu::Buffer,

    // camera
    pub camera : Camera,
    pub camera_controller : CameraController,
    pub camera_uniform : CameraUniform,
    pub camera_buffer : wgpu::Buffer,
    pub camera_bind_group : wgpu::BindGroup,

    pub clear_color : wgpu::Color,

    // imgui
    pub imgui_context : imgui::Context,
    pub last_frame : Instant,
    pub last_cursor : Option<imgui::MouseCursor>,
    pub renderer : Renderer,
    pub platform : imgui_winit_support::WinitPlatform,
    pub demo_open : bool,

    // layers
    pub layers : Vec<Layer>,
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
    pub async fn new(window : Window) -> Self {

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

        // NOTE: camera controller -> camera -> unifom -> buffer -> vextex shader

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
            usage : BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // @group(1) @binding(0) camera
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

        // NOTE: images
        // [doc] https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#loading-an-image-from-a-file

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
                    resource : TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding : 1,
                    resource : wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label : Some("diffuse_bind_group"),
        });

        // depth_texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // TODO: render_pipeline
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../assets/shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout], // NOTE:
                push_constant_ranges: &[],
            });

        use crate::model::ModelVertex;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label : Some("Render Pipeline"),
            layout : Some(&render_pipeline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",                               // 1.
                buffers : &[ModelVertex::desc(), InstanceRaw::desc()], // 2. added instances
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
            depth_stencil : None,
            // depth_stencil : Some(wgpu::DepthStencilState {
            //     format : texture::Texture::DEPTH_FORMAT,
            //     depth_write_enabled : true,
            //     depth_compare : wgpu::CompareFunction::Less, // 1.
            //     stencil : wgpu::StencilState::default(),     // 2.
            //     bias : wgpu::DepthBiasState::default(),
            // }),
            multisample : wgpu::MultisampleState {
                count : 1,                         // 2.
                mask : !0,                         // 3.
                alpha_to_coverage_enabled : false, // 4.
            },
            multiview : None, // 5.
        });

        // HACK: vertex buffer

        // let vertex_buffer_desc = &wgpu::util::BufferInitDescriptor {
        //     label : Some("Vertex Buffer"),
        //     contents : bytemuck::cast_slice(VERTICES),
        //     usage : wgpu::BufferUsages::VERTEX,
        // };
        //
        // let vertex_buffer = device.create_buffer_init(vertex_buffer_desc);
        //
        // HACK: index buffer

        // let index_buffer_desc = &wgpu::util::BufferInitDescriptor {
        //     label : Some("Index Buffer"),
        //     contents : bytemuck::cast_slice(INDICES),
        //     usage : wgpu::BufferUsages::INDEX,
        // };
        //
        // let index_buffer = device.create_buffer_init(index_buffer_desc);

        // let num_indices = INDICES.len() as u32;

        // ---------------------------------------------------------------------------------
        // NOTE: prepare imgui layers

        let mut layers = vec![];

        // ---------------------------------------------------------------------------------

        // NOTE: for window event detection, updating imgui state, etc

        let last_cursor = None;

        const SPACE_BETWEEN : f32 = 3.0;

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {

                (0..NUM_INSTANCES_PER_ROW).map(move |x| {

                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 {
                        x : x as f32,
                        y : 0.0,
                        z : z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {

                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {

                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Instance Buffer"),
            contents : bytemuck::cast_slice(&instance_data),
            usage : BufferUsages::VERTEX,
        });

        let obj_model =
            resource::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            // vertex_buffer,
            // index_buffer,
            // num_indices,
            instance_buffer,
            instances,
            diffuse_bind_group,
            diffuse_texture,
            depth_texture,
            obj_model,
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

            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            // TODO:

            let texture_context = &mut texture::Context {
                device : &self.device,
                queue : &self.queue,
                renderer : &mut self.renderer,
            };
        }
    }

    pub fn input(&mut self, event : &WindowEvent) -> bool {

        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {

        // update camera eye, target, fov,

        self.camera_controller.update_camera(&mut self.camera);

        // update v-p matrix from camera eye, target, fov, up

        self.camera_uniform.update_view_proj(&self.camera);

        // NOTE: camera.vp matrix -> slice -> uniform buffer -> shader
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

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

        let imgui_ui = self.imgui_context.frame();

        // NOTE: prepare imgui layers

        let _texture_context = &mut texture::Context {
            device : &self.device,
            queue : &self.queue,
            renderer : &mut self.renderer,
        };

        // self.imgui_render(texture_context, &mut imgui_ui);

        for _layer in &mut self.layers {

            if let Some(window) = imgui_ui
                .window("Gallery")
                .size([512.0, 512.0], imgui::Condition::FirstUseEver)
                .begin()
            {

                let _new_imgui_region_size = imgui_ui.content_region_avail();

                window.end();
            };
        }

        // NOTE: prepare render
        if self.last_cursor != imgui_ui.mouse_cursor() {

            self.last_cursor = imgui_ui.mouse_cursor();

            self.platform.prepare_render(&imgui_ui, &self.window);
        }

        let main_view = main_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut main_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label : Some("Render Encoder"),
                });

        // Render pass scope
        {

            let mut main_rpass = main_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                // depth_stencil_attachment : Some(wgpu::RenderPassDepthStencilAttachment {
                //     view : &self.depth_texture.view,
                //     depth_ops : Some(wgpu::Operations {
                //         load : wgpu::LoadOp::Clear(1.0),
                //         store : true,
                //     }),
                //     stencil_ops : None,
                // }),
            });

            main_rpass.set_vertex_buffer(1, self.instance_buffer.slice(..)); //NOTE: more instances

            main_rpass.set_pipeline(&self.render_pipeline);

            use crate::model::DrawModel;

            let mesh = &self.obj_model.meshes[0];

            let material = &self.obj_model.materials[mesh.material];

            main_rpass.draw_mesh_instanced(
                mesh,
                material,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
            );

            // NOTE: render imgui

            self.renderer
                .render(
                    self.imgui_context.render(),
                    &self.queue,
                    &self.device,
                    &mut main_rpass,
                )
                .expect("Render imgui failed");
        }

        // NOTE: submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(main_encoder.finish()));

        main_frame.present();

        Ok(())
    }
}
