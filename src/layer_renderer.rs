use crate::texture::*;
use image::*;

pub struct LayerRenderer {
    pub texture_id : imgui::TextureId,
    pub texture_texels : Vec<u8>,
    pub data : RgbaImage,
    pub size : Option<[f32; 2]>,
}

impl LayerRenderer {
    pub fn new(context : &mut Context, bytes : &[u8]) -> LayerRenderer {

        if bytes.len() == 0 {

            println!("Not source, creating default cube texture");

            // Create the texture
            let texture_size = 256u32;

            let texture_texels = Texture::create_texels(texture_size as usize);

            let texture = Texture::recreate_image(context, texture_size, &texture_texels);

            let texture_id = context.renderer.textures.insert(texture);

            let data = image::RgbaImage::new(256, 256);

            return LayerRenderer {
                texture_id,
                texture_texels,
                data,
                size : Some([256.0, 256.0]),
            };
        } else {

            println!("Got source, creating texture from file");

            let (image, size) = Texture::imgui_image_from_raw(bytes);

            let texture = Texture::imgui_texture_from_raw(context, &image, size);

            let texture_texels = image.to_rgba8().to_vec();

            let texture_id = context.renderer.textures.insert(texture);

            return LayerRenderer {
                texture_id,
                texture_texels,
                data : image.to_rgba8(),
                size : Some([size.width as f32, size.height as f32]),
            };
        }
    }

    pub fn render(&mut self, _context : &mut Context, ui : &imgui::Ui, size : Option<[f32; 2]>) {

        let [width, height] = size.unwrap();

        ui.invisible_button("Smooth Button", [width, height]);

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

    fn convert_color(color : wgpu::Color) -> u8 {

        let r = color.r as u8;

        let g = color.g as u8;

        let b = color.b as u8;

        let a = color.a as u8;

        let color1 = (r << 6) | (g << 4) | (b << 2) | a;

        color1
    }

    pub fn per_pixel(x : u32, y : u32) -> image::Rgba<u8> { image::Rgba([155, 155, 155, 1]) }
}
