use bytemuck::cast_slice;

use imgui::*;

use crate::texture;

pub struct LayerRenderer {
    pub texture : texture::Texture,
    pub data : Vec<u8>,
    pub height : u32,
    pub width : u32,
}

impl LayerRenderer {
    pub fn new(device : &wgpu::Device, queue : &wgpu::Queue, bytes : &[u8], path : &str) -> Self {

        let texture = texture::Texture::from_bytes(&device, &queue, bytes, path).unwrap(); // CHANGED!

        let width = texture.texture.width();

        let height = texture.texture.height();

        LayerRenderer {
            texture,
            data : bytes.to_vec(),
            height,
            width,
        }
    }

    pub fn from_bytes(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
        data : &[u8],
        label : &str,
    ) -> Self {

        let texture = texture::Texture::from_bytes(&device, &queue, data, label)
            .expect("Failed to load data"); // CHANGED!

        let width = texture.texture.width();

        let height = texture.texture.height();

        LayerRenderer {
            texture,
            data : data.to_vec(),
            height,
            width,
        }
    }

    pub fn resize() {}

    pub fn attach_image(&mut self, ui : &imgui::Ui) {

        {

            ui.invisible_button("Smooth Button", [100.0, 100.0]);

            let draw_list = ui.get_window_draw_list();

            // draw_list
            //     .add_image_rounded(
            //         self.texture.texture.into,
            //         ui.item_rect_min(),
            //         ui.item_rect_max(),
            //         16.0,
            //     ) // Tint brighter for
            //     .col([2.0, 0.5, 0.5, 1.0])
            //     // Rounding on each corner can be changed separately
            //     .round_top_left(ui.frame_count() / 60 % 4 == 0)
            //     .round_top_right((ui.frame_count() + 1) / 60 % 4 == 1)
            //     .round_bot_right((ui.frame_count() + 3) / 60 % 4 == 2)
            //     .round_bot_left((ui.frame_count() + 2) / 60 % 4 == 3)
            //     .build();
        }
    }

    pub fn render(&mut self, device : &wgpu::Device, queue : &wgpu::Queue) {

        //
        for y in 0..self.height {

            for x in 0..self.width {

                // Insert RGB values
                self.data.push(y as u8);

                self.data.push(x as u8);

                self.data.push((y + x) as u8);

                self.data.push(1.0 as u8);
            }
        }

        // for y in 0..self.height as usize {
        //
        //     for x in 0..self.width as usize {
        //
        //         let color = Self::per_pixel(x as u32, y as u32);
        //
        //         let index = x + y * self.width as usize;
        //
        //         if index < self.width as usize * self.height as usize {
        //
        //             data[index] = Self::convert_color(color);
        //         }
        //     }
        // }

        Self::from_bytes(device, queue, &self.data, "test");
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
