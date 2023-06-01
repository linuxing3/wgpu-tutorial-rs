use crate::{
    camera::{Camera, CameraController, CameraUniform},
    model::Model,
    resource,
    share::create_empty_texels,
};
use std::mem;
use wgpu::{include_wgsl, util::DeviceExt, BufferUsages};
use winit::event::WindowEvent;

use crate::share::{create_cube_texels, create_vertices, ImVertex, OPENGL_TO_WGPU_MATRIX};

pub struct Swapchain {
    pub vertex_buf : wgpu::Buffer,
    pub index_buf : wgpu::Buffer,
    pub index_count : usize,
    pub camera_bind_group : wgpu::BindGroup,
    pub texture_bind_group : wgpu::BindGroup,
    pub pipeline : wgpu::RenderPipeline,
    pub obj_model : Option<Model>,
    // camera
    pub uniform_buf : wgpu::Buffer,
    pub camera : Camera,
    pub camera_controller : CameraController,
    pub camera_uniform : CameraUniform,
    pub time : f32,
}

impl Swapchain {
    pub fn new(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) -> Self {

        //vertex index
        let (vertex_data, index_data) = create_vertices();

        let index_count = index_data.len();

        let (vertex_buf, index_buf) = Self::configure_vertex(device, vertex_data, index_data);

        // NOTE: texture underlay, data copied

        let texture_size = 256u32;

        let texture_texels = create_empty_texels(texture_size as usize, texture_size as usize);

        let cube_texture_extent = wgpu::Extent3d {
            width : texture_size,
            height : texture_size,
            depth_or_array_layers : 1,
        };

        let (_, cube_texture_view, cube_texture_sampler) = Self::configure_texture(
            device,
            queue,
            texture_size,
            texture_texels,
            cube_texture_extent,
        );

        // uniform

        let time = 0.0;

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

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Camera Buffer"),
            contents : bytemuck::cast_slice(&[camera_uniform]),
            usage : BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let (camera_bind_group, camera_bind_group_layout) =
            Self::configure_camera_bind_group(device, &uniform_buf);

        let (texture_bind_group, texture_bind_group_layout) =
            Self::configure_texture_bind_group(device, &cube_texture_view, &cube_texture_sampler);

        // pipeline
        let pipeline = Self::configure_pipeline(
            config,
            device,
            &[&camera_bind_group_layout, &texture_bind_group_layout],
        );

        // Done
        Swapchain {
            vertex_buf,
            index_buf,
            index_count,
            camera_bind_group,
            texture_bind_group,
            obj_model : None,
            uniform_buf,
            camera,
            camera_uniform,
            camera_controller,
            pipeline,
            time,
        }
    }

    fn configure_vertex(
        device : &wgpu::Device,
        vertex_data : Vec<ImVertex>,
        index_data : Vec<u16>,
    ) -> (wgpu::Buffer, wgpu::Buffer) {

        // Create the vertex and index buffers

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

        (vertex_buf, index_buf)
    }

    fn configure_texture_from_image(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {

        let bytes = include_bytes!("../assets/images/happy-tree.png");

        let img = image::load_from_memory(bytes).unwrap();

        let rgba = img.to_rgba8();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label : Some("texture"),
            size : wgpu::Extent3d {
                width : img.width(),
                height : img.height(),
                depth_or_array_layers : 1,
            },
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format : wgpu::TextureFormat::Rgba8UnormSrgb,
            usage : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats : &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &rgba,
            wgpu::ImageDataLayout {
                offset : 0,
                bytes_per_row : std::num::NonZeroU32::new(4 * img.width()),
                rows_per_image : std::num::NonZeroU32::new(img.height()),
            },
            wgpu::Extent3d {
                width : img.width(),
                height : img.height(),
                depth_or_array_layers : 1,
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u : wgpu::AddressMode::ClampToEdge,
            address_mode_v : wgpu::AddressMode::ClampToEdge,
            address_mode_w : wgpu::AddressMode::ClampToEdge,
            mag_filter : wgpu::FilterMode::Linear,
            min_filter : wgpu::FilterMode::Nearest,
            mipmap_filter : wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        (texture, texture_view, texture_sampler)
    }

    fn configure_texture(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
        texture_size : u32,
        texture_texels : Vec<u8>,
        cube_texture_extent : wgpu::Extent3d,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {

        // here texture as copy destination
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

        let cube_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u : wgpu::AddressMode::ClampToEdge,
            address_mode_v : wgpu::AddressMode::ClampToEdge,
            address_mode_w : wgpu::AddressMode::ClampToEdge,
            mag_filter : wgpu::FilterMode::Linear,
            min_filter : wgpu::FilterMode::Linear,
            mipmap_filter : wgpu::FilterMode::Nearest,
            compare : Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp : 0.0,
            lod_max_clamp : 100.0,
            ..Default::default()
        });

        // HACK: copy data into texture
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

        (cube_texture, cube_texture_view, cube_texture_sampler)
    }

    pub fn handle_input(&mut self, event : &WindowEvent) -> bool {

        self.camera_controller.process_events(event)
    }

    fn configure_camera_bind_group(
        device : &wgpu::Device,
        uniform_buf : &wgpu::Buffer,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label : None,
                entries : &[wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::VERTEX,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset : false,
                        min_binding_size : wgpu::BufferSize::new(64),
                    },
                    count : None,
                }],
            });

        // Create bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &camera_bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : uniform_buf.as_entire_binding(),
            }],
            label : None,
        });

        (camera_bind_group, camera_bind_group_layout)
    }

    fn configure_texture_sampler_bind_group(
        device : &wgpu::Device,
        cube_texture_view : &wgpu::TextureView,
        cube_texture_sampler : &wgpu::Sampler,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {

        // Create pipeline layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label : None,
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
                        ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count : None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &texture_bind_group_layout,
            entries : &[
                wgpu::BindGroupEntry {
                    binding : 0,
                    resource : wgpu::BindingResource::TextureView(&cube_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding : 1,
                    resource : wgpu::BindingResource::Sampler(&cube_texture_sampler),
                },
            ],
            label : None,
        });

        (texture_bind_group, texture_bind_group_layout)
    }

    fn configure_texture_bind_group(
        device : &wgpu::Device,
        cube_texture_view : &wgpu::TextureView,
        cube_texture_sampler : &wgpu::Sampler,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {

        // Create pipeline layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label : None,
                entries : &[wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Texture {
                        multisampled : false,
                        view_dimension : wgpu::TextureViewDimension::D2,
                        sample_type : wgpu::TextureSampleType::Uint, // BUG:
                    },
                    count : None,
                }],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &texture_bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : wgpu::BindingResource::TextureView(&cube_texture_view),
            }],
            label : None,
        });

        (texture_bind_group, texture_bind_group_layout)
    }

    fn configure_pipeline_with_model(
        device : &wgpu::Device,
        bind_group_layouts : &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : None,
            bind_group_layouts,
            push_constant_ranges : &[],
        });

        let shader = device.create_shader_module(include_wgsl!("../assets/shaders/shader.wgsl"));

        use crate::model::ModelVertex;

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label : None,
            layout : Some(&pipeline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers : &[ModelVertex::desc()],
            },
            fragment : Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                // targets : &[Some(config.format.into())],
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
            multisample : wgpu::MultisampleState::default(),
            multiview : None,
        });

        pipeline
    }

    fn configure_pipeline(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
        bind_group_layouts : &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<ImVertex>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : None,
            bind_group_layouts,
            push_constant_ranges : &[],
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

        pipeline
    }

    pub fn update(&mut self, delta_time : f32) { self.time += delta_time; }

    // HACK:

    pub fn setup_camera(&mut self, queue : &wgpu::Queue, _size : [f32; 2]) {

        // update camera eye, target, fov,

        self.camera_controller.update_camera(&mut self.camera);

        // update v-p matrix from camera eye, target, fov, up

        self.camera_uniform.update_view_proj(&self.camera);

        // NOTE: camera.vp matrix -> slice -> uniform buffer -> shader
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

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

    pub async fn load_model(&mut self, device : &wgpu::Device, queue : &wgpu::Queue) {

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

        let obj_model =
            resource::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        self.obj_model = Some(obj_model);
    }
}
