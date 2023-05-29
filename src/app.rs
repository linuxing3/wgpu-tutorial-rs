use wgpu::{include_wgsl, util::DeviceExt};

use crate::share::*;

pub struct State {
    vertex_buf : wgpu::Buffer,
    index_buf : wgpu::Buffer,
    index_count : usize,
    bind_group : wgpu::BindGroup,
    uniform_buf : wgpu::Buffer,
    pipeline : wgpu::RenderPipeline,
    time : f32,
}

impl State {
    pub fn generate_matrix(aspect_ratio : f32) -> cgmath::Matrix4<f32> {

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
    pub fn init(
        config : &wgpu::SurfaceConfiguration,
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) -> Self {

        use std::mem;

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<ImVertex>();

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
        //
        // 1. Vertex entry
        // struct Locals {
        //     transform: mat4x4<f32>,
        // };
        // @group(0) @binding(0)
        // var<uniform> r_locals: Locals;

        // 2. texture entry
        // @group(0) @binding(1)
        // var r_color: texture_2d<u32>;
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

        let texture_texels = create_cube_texels(texture_size as usize);

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

    pub fn update(&mut self, delta_time : f32) { self.time += delta_time; }

    pub fn setup_camera(&mut self, queue : &wgpu::Queue, size : [f32; 2]) {

        let mx_total = Self::generate_matrix(size[0] / size[1]);

        let mx_ref : &[f32; 16] = mx_total.as_ref();

        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    pub fn imgui_render(
        &mut self,
        view : &wgpu::TextureView,
        device : &wgpu::Device,
        queue : &wgpu::Queue,
    ) {

        let mut imgui_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label : None });

        {

            let mut imgui_rpass = imgui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            imgui_rpass.push_debug_group("Prepare data for draw.");

            imgui_rpass.set_pipeline(&self.pipeline);

            imgui_rpass.set_bind_group(0, &self.bind_group, &[]);

            imgui_rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);

            imgui_rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));

            imgui_rpass.pop_debug_group();

            imgui_rpass.insert_debug_marker("Draw!");

            imgui_rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        queue.submit(Some(imgui_encoder.finish()));
    }
}
