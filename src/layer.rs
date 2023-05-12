use crate::layer_renderer::LayerRenderer;
use crate::texture;
use imgui::*;
use std::time::Instant;

use crate::State;

use crate::Camera;

pub struct Layer {
    pub renderer: LayerRenderer,
    pub last_frame: Instant,
}

impl Layer {
    pub fn new(
        context: &mut texture::Context,
        bytes: &[u8],
    ) -> Layer {
        let renderer = LayerRenderer::new(context, bytes);

        let last_frame = Instant::now();

        Layer {
            renderer,
            last_frame,
        }
    }

    pub fn update(
        &mut self,
        width: f32,
        height: f32,
    ) {
        self.renderer.update(width, height);
    }

    pub fn render(
        &mut self,
        context: &mut texture::Context,
        ui: &imgui::Ui,
    ) {
        let now = Instant::now();

        self.last_frame = now;

        self.renderer.set_data(context);

        self.renderer.render(ui);
    }

    pub fn renderer(&self) -> &LayerRenderer {
        &self.renderer
    }
}
