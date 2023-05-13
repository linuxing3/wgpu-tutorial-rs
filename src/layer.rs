use crate::layer_renderer::LayerRenderer;
use crate::texture;
use std::time::Instant;

pub struct Layer {
    pub renderer : LayerRenderer,
    pub last_frame : Instant,
}

impl Layer {
    pub fn new(context : &mut texture::Context, data : &[u8]) -> Layer {

        // let data : [u8; 0] = [];

        let renderer = LayerRenderer::new(context, &data);

        let last_frame = Instant::now();

        Layer {
            renderer,
            last_frame,
        }
    }

    pub fn render(
        &mut self,
        context : &mut texture::Context,
        ui : &imgui::Ui,
        size : Option<[f32; 2]>,
    ) {

        self.renderer.render2(context, ui, size);
    }

    pub fn renderer(&self) -> &LayerRenderer { &self.renderer }
}
