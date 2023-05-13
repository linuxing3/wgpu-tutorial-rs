use crate::texture::*;
use image::*;

pub struct LayerRenderer {
    pub texture_id : imgui::TextureId,
    pub data : RgbaImage,
    pub size : Option<[f32; 2]>,
}

impl LayerRenderer {
    pub fn new(context : &mut Context, bytes : &[u8]) -> LayerRenderer {

        if bytes.len() == 0 {

            let size = wgpu::Extent3d {
                width : 256,
                height : 256,
                ..Default::default()
            };

            let image = DynamicImage::new_rgba8(size.width, size.height);

            let texture = Texture::imgui_texture_from_raw(context, &image, size);

            // BUG:
            let texture_id = context.renderer.textures.insert(texture);

            return LayerRenderer {
                texture_id,
                data : image.to_rgba8(),
                size : Some([512.0, 512.0]),
            };
        } else {

            let (image, size) = Texture::imgui_image_from_raw(bytes);

            let texture = Texture::imgui_texture_from_raw(context, &image, size);

            // BUG:
            let texture_id = context.renderer.textures.insert(texture);

            return LayerRenderer {
                texture_id,
                data : image.to_rgba8(),
                size : Some([size.width as f32, size.height as f32]),
            };
        }
    }

    pub fn render(&mut self, ui : &imgui::Ui) { self.build_image(ui); }

    pub fn build_image(&mut self, ui : &imgui::Ui) {

        let [width, height] = self.size.unwrap();

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

    pub fn set_data(&mut self, context : &mut Context, _ui : &imgui::Ui) {

        let [width, height] = self.size.unwrap();

        for y in 0..height as u32 {

            for x in 0..width as u32 {

                let color = Self::per_pixel(x as u32, y as u32);

                self.data.put_pixel(x, y, color);
            }
        }

        let raw_data = self.data.clone().into_raw();

        let size = wgpu::Extent3d {
            width : width as u32,
            height : height as u32,
            ..Default::default()
        };

        let texture_config = imgui_wgpu::TextureConfig {
            size,
            label : Some("raw texture"),
            format : Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(&context.device, &context.renderer, texture_config);

        texture.write(&context.queue, &raw_data, width as u32, height as u32);

        self.texture_id = context.renderer.textures.insert(texture);
    }

    pub fn resize(&mut self, context : &mut Context, ui : &imgui::Ui, size : Option<[f32; 2]>) {

        let imgui_region_size = size.unwrap();

        let scale = &ui.io().display_framebuffer_scale;

        let texture_config = imgui_wgpu::TextureConfig {
            size : wgpu::Extent3d {
                width : (imgui_region_size[0] * scale[0]) as u32,
                height : (imgui_region_size[1] * scale[1]) as u32,
                ..Default::default()
            },
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ..Default::default()
        };

        println!("w:{}    h:{}", imgui_region_size[0], imgui_region_size[1]);

        context.renderer.textures.replace(
            self.texture_id,
            imgui_wgpu::Texture::new(&context.device, &context.renderer, texture_config),
        );
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
