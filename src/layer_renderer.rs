use imgui::*;

use crate::texture;
use image::ImageFormat;
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use std::time::Instant;
use wgpu::Extent3d;

pub struct LayerRenderer {
    pub texture_id : imgui::TextureId,
    pub image : imgui::Image,
    pub data : Vec<u8>,
    pub height : u32,
    pub width : u32,
}

impl LayerRenderer {
    pub fn new(texture_id : imgui::TextureId) -> Self {

        let data : Vec<u8> = Vec::with_capacity(0);

        let image = imgui::Image::new(texture_id, [0.0, 0.0]);

        LayerRenderer {
            texture_id,
            image,
            data,
            height : 0,
            width : 0,
        }
    }

    pub fn render(&mut self, ui : &imgui::Ui) {

        // Self::allocate_memory(self, ui);

        Self::set_data(self);
    }

    pub fn allocate_memory(&mut self, ui : &imgui::Ui) {

        ui.invisible_button("Smooth Button", [100.0, 100.0]);

        let draw_list = ui.get_window_draw_list();

        draw_list
            .add_image_rounded(
                self.texture_id,
                ui.item_rect_min(),
                ui.item_rect_max(),
                16.0,
            ) // Tint brighter for
            .col([2.0, 0.5, 0.5, 1.0])
            // Rounding on each corner can be changed separately
            .round_top_left(ui.frame_count() / 60 % 4 == 0)
            .round_top_right((ui.frame_count() + 1) / 60 % 4 == 1)
            .round_bot_right((ui.frame_count() + 3) / 60 % 4 == 2)
            .round_bot_left((ui.frame_count() + 2) / 60 % 4 == 3)
            .build();
    }

    pub fn set_data(&mut self) {

        let len = self.width as usize * self.height as usize;

        for y in 0..self.height as usize {

            for x in 0..self.width as usize {

                let color = Self::per_pixel(x as u32, y as u32);

                let index = x + y * self.width as usize;

                if index < len {

                    self.data[index] = Self::convert_color(color);
                }
            }
        }

        // for y in 0..self.height {
        //
        //     for x in 0..self.width {
        //
        //         // Insert RGB values
        //         data.push(y as u8);
        //
        //         data.push(x as u8);
        //
        //         data.push((y + x) as u8);
        //
        //         data.push(1.0 as u8);
        //     }
        // }
        // self.data = data.to_vec();
    }

    pub fn resize(&mut self, width : u32, height : u32) {

        if self.width != width || self.height != height {

            self.width = width;

            self.height = height;
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
