use crate::texture::{Context, Texture};
use imgui::{TextureId, Ui};

pub struct Layer {
    texture_id : TextureId,
    size : Option<[f32; 2]>,
}

impl Layer {
    pub fn new(texture_id : TextureId, size : [f32; 2]) -> Layer {

        println!("texture id is: {}", texture_id.id());

        Layer {
            texture_id,
            size : Some(size),
        }
    }

    pub fn size(&mut self) -> Option<[f32; 2]> { self.size }

    pub fn set_size(&mut self, size : u32) { self.size = Some([size as f32, size as f32]); }

    pub fn id(&mut self) -> TextureId { self.texture_id }

    fn set_bytes(&mut self, context : &mut Context, bytes : &[u8]) {

        println!("Got source, setting texture from file");

        let (image, size) = Texture::imgui_image_from_raw(bytes);

        let texture = Texture::imgui_texture_from_raw(context, &image, size);

        self.size = Some([size.width as f32, size.height as f32]);

        context.renderer.textures.replace(self.texture_id, texture);
    }

    fn set_texels(&mut self, context : &mut Context, size : u32, texture_texels : Vec<u8>) {

        let texture = Texture::recreate_image(context, size, &texture_texels);

        self.size = Some([size as f32, size as f32]);

        context.renderer.textures.replace(self.texture_id, texture);
    }

    pub fn render(&mut self, _context : &mut Context, ui : &mut Ui, title : &str) {

        ui.window(title)
            .size(self.size().unwrap(), imgui::Condition::FirstUseEver)
            .build(|| {

                self.size = Some(ui.content_region_avail());

                imgui::Image::new(self.texture_id, self.size.unwrap()).build(ui);
            });
    }

    pub fn resize(&mut self, context : &mut Context, ui : &Ui, new_size : [f32; 2]) {

        let scale = &ui.io().display_framebuffer_scale;

        let texture_config = imgui_wgpu::TextureConfig {
            size : wgpu::Extent3d {
                width : (new_size[0] * scale[0]) as u32,
                height : (new_size[1] * scale[1]) as u32,
                ..Default::default()
            },
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ..Default::default()
        };

        context.renderer.textures.replace(
            self.texture_id,
            imgui_wgpu::Texture::new(&context.device, &context.renderer, texture_config),
        );
    }
}
