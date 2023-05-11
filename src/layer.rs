use crate::layer_renderer::LayerRenderer;
use imgui::*;
use std::time::Instant;

use crate::State;

use crate::Camera;

pub struct Layer {
    pub renderer : LayerRenderer,
    pub last_frame : Instant,
}

impl Layer {
    pub fn new(id : imgui::TextureId) -> Layer {

        let renderer = LayerRenderer::new(id);

        let last_frame = Instant::now();

        Layer {
            renderer,
            last_frame,
        }
    }

    pub fn attach_text(&mut self, ui : &imgui::Ui, text : &str) { ui.text(text); }

    pub fn on_dettach(&mut self) {

        unimplemented!();
    }

    pub fn update() {

        unimplemented!();
    }

    pub fn render(&mut self, ui : &imgui::Ui) {

        let now = Instant::now();

        self.last_frame = now;

        self.renderer.render(ui);
    }
}
