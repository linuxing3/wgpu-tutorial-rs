use crate::texture::*;
use image::*;

pub struct LayerRenderer {
    pub texture_id: imgui::TextureId,
    pub data: RgbaImage,
    pub height: u32,
    pub width: u32,
}

impl LayerRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
        bytes: &[u8],
    ) -> LayerRenderer {
        let (image, size) = Texture::imgui_image_from_raw(bytes);

        let texture = Texture::imgui_texture_from_raw(device, queue, renderer, &image, size);

        let texture_id = renderer.textures.insert(texture);

        LayerRenderer {
            texture_id,
            data: image.to_rgba8(),
            height: size.height,
            width: size.width,
        }
    }

    pub fn render(
        &mut self,
        ui: &imgui::Ui,
    ) {
        self.push_to_command_list(ui);
    }

    pub fn push_to_command_list(
        &mut self,
        ui: &imgui::Ui,
    ) {
        ui.invisible_button("Smooth Button", [self.height as f32, self.height as f32]);

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

    pub fn set_data(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut imgui_wgpu::Renderer,
    ) {
        let (width, height) = (self.width, self.height);

        for y in height / 4..height * 3 / 4 {
            for x in width / 4..width * 3 / 4 {
                let color = Self::per_pixel(x as u32, y as u32);

                self.data.put_pixel(x, y, color);
            }
        }

        let raw_data = self.data.clone().into_raw();

        let size = wgpu::Extent3d {
            width,
            height,
            ..Default::default()
        };

        let texture_config = imgui_wgpu::TextureConfig {
            size,
            label: Some("raw texture"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };

        let texture = imgui_wgpu::Texture::new(&device, &renderer, texture_config);

        texture.write(&queue, &raw_data, width, height);

        self.texture_id = renderer.textures.insert(texture);
    }

    pub fn update(
        &mut self,
        width: f32,
        height: f32,
    ) {

        // if self.data.width() != width | self.data.height() != height {
        //
        //
        //
        // }
    }

    fn convert_color(color: wgpu::Color) -> u8 {
        let r = color.r as u8;

        let g = color.g as u8;

        let b = color.b as u8;

        let a = color.a as u8;

        let color1 = (r << 6) | (g << 4) | (b << 2) | a;

        color1
    }

    pub fn per_pixel(
        x: u32,
        y: u32,
    ) -> image::Rgba<u8> {
        image::Rgba([155, 155, 155, 1])
    }
}
