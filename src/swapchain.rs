use std::mem;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::share::{create_cube_texels, create_vertices, ImVertex, OPENGL_TO_WGPU_MATRIX};

pub struct Swapchain {
    pub vertex_buffer : wgpu::Buffer,
    pub index_buffer : wgpu::Buffer,
    pub index_count : usize,
    pub bind_group : wgpu::BindGroup,
    pub uniform_buffer : wgpu::Buffer,
    pub render_pipeline : wgpu::RenderPipeline,
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

        let (vertex_buffer, index_buffer) = Self::configure_vertex(device, vertex_data, index_data);

        // texture

        let texture_size = 256u32;

        let texture_texels = create_cube_texels(texture_size as usize);

        let cube_texture_extent = wgpu::Extent3d {
            width : texture_size,
            height : texture_size,
            depth_or_array_layers : 1,
        };

        let (_, cube_texture_view) = Self::configure_texture(
            device,
            queue,
            texture_size,
            texture_texels,
            cube_texture_extent,
        );

        // uniform
        let uniform_buffer = Self::configure_uniform(config, device);

        let (bind_group, bind_group_layout) =
            Self::configure_bind_group(device, &uniform_buffer, &cube_texture_view);

        // pipeline
        let render_pipeline = Self::configure_pipeline(config, device, &bind_group_layout);

        // Done
        Swapchain {
            vertex_buffer,
            index_buffer,
            index_count,
            bind_group,
            uniform_buffer,
            render_pipeline,
            time : 0.0,
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

    fn configure_texture(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
        texture_size : u32,
        texture_texels : Vec<u8>,
        cube_texture_extent : wgpu::Extent3d,
    ) -> (wgpu::Texture, wgpu::TextureView) {

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

        (cube_texture, cube_texture_view)
    }

    fn configure_uniform(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
    ) -> wgpu::Buffer {

        // Create other resources
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);

        let mx_ref : &[f32; 16] = mx_total.as_ref();

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Uniform Buffer"),
            contents : bytemuck::cast_slice(mx_ref),
            usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        uniform_buf
    }

    fn configure_bind_group(
        device : &wgpu::Device,
        uniform_buf : &wgpu::Buffer,
        cube_texture_view : &wgpu::TextureView,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {

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

        (bind_group, bind_group_layout)
    }

    fn configure_pipeline(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
        bind_group_layout : &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<ImVertex>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : None,
            bind_group_layouts : &[&bind_group_layout],
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

    // not has_dynamic_effect
    fn generate_matrix(aspect_ratio : f32) -> cgmath::Matrix4<f32> {

        // TODO:

        // let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect,
        // self.znear, self.zfar);
        // let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);

        let mx_view = cgmath::Matrix4::look_at_rh(
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );

        let mx_correction = OPENGL_TO_WGPU_MATRIX;

        mx_correction * mx_projection * mx_view
    }

    pub fn update(&mut self, delta_time : f32) { self.time += delta_time; }

    pub fn setup_camera(&mut self, queue : &wgpu::Queue, size : [f32; 2]) {

        let mx_total = Self::generate_matrix(size[0] / size[1]);

        let mx_ref : &[f32; 16] = mx_total.as_ref();

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(mx_ref));
    }

    pub fn render(
        &mut self,
        view : &wgpu::TextureView,
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) {

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

            rpass.set_pipeline(&self.render_pipeline);

            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            rpass.pop_debug_group();

            rpass.insert_debug_marker("Draw!");

            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
