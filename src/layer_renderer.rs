use bytemuck::cast_slice;

use imgui::*;

use crate::texture;

// #[derive(Debug)]
// pub struct Texture {
//     context: Arc<C>,
//     id: ObjectId,
//     data: Box<Data>,
//     owned: bool,
//     descriptor: TextureDescriptor<'static>,
// }
// Any is a static trait
// type Data = dyn Any + Send + Sync;

pub struct LayerRenderer {
    pub texture : texture::Texture,
    pub height : u32,
    pub width : u32,
}

impl LayerRenderer {
    pub fn new(device : &wgpu::Device, queue : &wgpu::Queue, bytes : &[u8], path : &str) -> Self {

        let _texture = texture::Texture::from_bytes(&device, &queue, bytes, path).unwrap(); // CHANGED!

        let _width = _texture.texture.width();

        let _height = _texture.texture.height();

        LayerRenderer {
            texture : _texture,
            height : _height,
            width : _width,
        }
    }

    pub fn from_bytes(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
        data : &[u8],
        label : &str,
    ) -> Self {

        let _texture = texture::Texture::from_bytes(&device, &queue, data, label).unwrap(); // CHANGED!

        let _width = _texture.texture.width();

        let _height = _texture.texture.height();

        LayerRenderer {
            texture : _texture,
            height : _height,
            width : _width,
        }
    }

    pub fn resize() {}

    pub fn render(&mut self, data : &mut [u8]) {

        for y in 0..self.height as usize {

            for x in 0..self.width as usize {

                let color = Self::per_pixel(x as u32, y as u32);

                data[x + y * self.width as usize] = Self::convert_color(color);
            }
        }
    }

    fn convert_color(color : wgpu::Color) -> u8 {

        let r = color.r as u8;

        let g = color.g as u8;

        let b = color.b as u8;

        let a = color.a as u8;

        let color1 = (r << 6) | (g << 4) | (b << 2) | a;

        color1
    }

    pub fn per_pixel(x : u32, y : u32) -> wgpu::Color {

        wgpu::Color {
            r : 0.0,
            g : 0.1,
            b : 0.2,
            a : 0.3,
        }
    }
}
