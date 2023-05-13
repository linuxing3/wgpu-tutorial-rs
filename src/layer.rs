use crate::layer_renderer::LayerRenderer;
use crate::texture;
use std::time::Instant;

pub struct Layer {
    pub renderer : LayerRenderer,
    pub last_frame : Instant,
}

impl Layer {
    pub fn new(context : &mut texture::Context, bytes : &[u8]) -> Layer {

        let renderer = LayerRenderer::new(context, bytes);

        let last_frame = Instant::now();

        Layer {
            renderer,
            last_frame,
        }
    }

    pub fn render(&mut self, context : &mut texture::Context, ui : &imgui::Ui) {

        self.renderer.set_data(context, ui);

        self.renderer.render(ui);
    }

    pub fn resize(
        &mut self,
        context : &mut texture::Context,
        ui : &imgui::Ui,
        size : Option<[f32; 2]>,
    ) {

        self.renderer.resize(context, ui, size);
    }

    pub fn renderer(&self) -> &LayerRenderer { &self.renderer }
}
