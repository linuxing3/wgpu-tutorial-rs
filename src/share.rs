use bytemuck;

pub const OPENGL_TO_WGPU_MATRIX : cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

pub struct VertexBasic {
    position : [f32; 3],
    color : [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

pub struct Vertex {
    position : [f32; 3],
    tex_coords : [f32; 2], // NEW!
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {

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

pub const VERTICES : &[Vertex] = &[
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

pub const INDICES : &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

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

// Example code modified from https://github.com/gfx-rs/wgpu-rs/tree/master/examples/cube
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]

pub struct ImVertex {
    pub pos : [f32; 4],
    pub tex_coord : [f32; 2],
}

pub fn imvertex(pos : [i8; 3], tc : [i8; 2]) -> ImVertex {

    ImVertex {
        pos : [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        tex_coord : [tc[0] as f32, tc[1] as f32],
    }
}

pub fn create_vertices() -> (Vec<ImVertex>, Vec<u16>) {

    let vertex_data = [
        // top (0, 0, 1)
        imvertex([-1, -1, 1], [0, 0]),
        imvertex([1, -1, 1], [1, 0]),
        imvertex([1, 1, 1], [1, 1]),
        imvertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        imvertex([-1, 1, -1], [1, 0]),
        imvertex([1, 1, -1], [0, 0]),
        imvertex([1, -1, -1], [0, 1]),
        imvertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        imvertex([1, -1, -1], [0, 0]),
        imvertex([1, 1, -1], [1, 0]),
        imvertex([1, 1, 1], [1, 1]),
        imvertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        imvertex([-1, -1, 1], [1, 0]),
        imvertex([-1, 1, 1], [0, 0]),
        imvertex([-1, 1, -1], [0, 1]),
        imvertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        imvertex([1, 1, -1], [1, 0]),
        imvertex([-1, 1, -1], [0, 0]),
        imvertex([-1, 1, 1], [0, 1]),
        imvertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        imvertex([1, -1, 1], [0, 0]),
        imvertex([-1, -1, 1], [1, 0]),
        imvertex([-1, -1, -1], [1, 1]),
        imvertex([1, -1, -1], [0, 1]),
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

pub fn create_cube_texels(size : usize) -> Vec<u8> {

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
