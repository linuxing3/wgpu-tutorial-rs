use bytemuck;

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
