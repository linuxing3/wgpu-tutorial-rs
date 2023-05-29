use crate::texture::{self, *};
use image::*;
use imgui_wgpu::TextureConfig;
use wgpu::{Extent3d, TextureUsages};

pub struct LayerRenderer {
    pub texture_id : imgui::TextureId,
    pub texture_texels : Vec<u8>,
    pub image_data : RgbaImage,
    pub size : [f32; 2],
}

impl LayerRenderer {
    pub fn new() -> LayerRenderer {

        println!("Got source, creating texture from file");

        let image = image::DynamicImage::new_rgba8(0, 0);

        let texture_texels = vec![];

        let texture_id = imgui::TextureId::new(0);

        LayerRenderer {
            texture_id,
            texture_texels,
            image_data : image.to_rgba8(),
            size : [0.0, 0.0],
        }
    }

    pub fn set_bytes(&mut self, context : &mut Context, bytes : &[u8]) {

        println!("Got source, setting texture from file");

        let (image, size) = Texture::imgui_image_from_raw(bytes);

        let texture = Texture::imgui_texture_from_raw(context, &image, size);

        self.size = [size.width as f32, size.height as f32];

        context.renderer.textures.replace(self.texture_id, texture);
    }

    pub fn set_texels(&mut self, context : &mut Context, size : u32, texture_texels : Vec<u8>) {

        let texture = texture::Texture::recreate_image(context, size, &texture_texels);

        self.size = [size as f32, size as f32];

        context.renderer.textures.replace(self.texture_id, texture);
    }

    pub fn resize(&mut self, context : &mut Context, ui : &imgui::Ui, new_size : [f32; 2]) {

        if let Some(_size) = Some(new_size) {

            // Resize render target, which is optional
            if _size != self.size && _size[0] >= 1.0 && _size[1] >= 1.0 {

                let scale = &ui.io().display_framebuffer_scale;

                let texture_config = TextureConfig {
                    size : Extent3d {
                        width : (self.size[0] * scale[0]) as u32,
                        height : (self.size[1] * scale[1]) as u32,
                        ..Default::default()
                    },
                    usage : TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    ..Default::default()
                };

                context.renderer.textures.replace(
                    self.texture_id,
                    imgui_wgpu::Texture::new(&context.device, &context.renderer, texture_config),
                );
            }
        }
    }

    pub fn render(&mut self, _context : &mut Context, ui : &imgui::Ui, size : [f32; 2]) {

        let [width, height] = size;

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

    pub fn render2(&mut self, _context : &mut Context, ui : &imgui::Ui, size : Option<[f32; 2]>) {

        imgui::Image::new(self.texture_id, size.unwrap()).build(ui);
    }

    fn convert_color(color : wgpu::Color) -> u8 {

        let r = color.r as u8;

        let g = color.g as u8;

        let b = color.b as u8;

        let a = color.a as u8;

        let color1 = (r << 6) | (g << 4) | (b << 2) | a;

        color1
    }

    pub fn per_pixel(_x : u32, _y : u32) -> image::Rgba<u8> { image::Rgba([155, 155, 155, 1]) }
}
