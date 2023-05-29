use crate::layer_renderer::LayerRenderer;
use crate::texture::{Context, Texture};
use imgui::Ui;
use std::time::Instant;

pub struct Layer {
    pub renderer : LayerRenderer,
    pub last_frame : Instant,
}

impl Layer {
    pub fn new(context : &mut Context, data : &[u8]) -> Layer {

        let last_frame = Instant::now();

        let mut renderer = LayerRenderer::new();

        renderer.set_bytes(context, data);

        Layer {
            renderer,
            last_frame,
        }
    }

    pub fn set_texels(&mut self, context : &mut Context, texture_size : u32) {

        // Create the texture
        let texture_texels = Texture::create_texels(texture_size as usize);

        self.renderer
            .set_texels(context, texture_size, texture_texels);
    }

    pub fn render(&mut self, context : &mut Context, ui : &Ui, size : [f32; 2]) {

        self.renderer.render(context, ui, size);
    }

    pub fn resize(&mut self, context : &mut Context, ui : &Ui, new_size : [f32; 2]) {

        self.renderer.resize(context, ui, new_size);
    }

    pub fn renderer(&self) -> &LayerRenderer { &self.renderer }
}
