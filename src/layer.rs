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
    pub fn new(
        device : &wgpu::Device,
        queue : &wgpu::Queue,
        renderer : &mut imgui_wgpu::Renderer,
        bytes : &[u8],
    ) -> Layer {

        let renderer = LayerRenderer::new(&device, &queue, renderer, bytes);

        let last_frame = Instant::now();

        Layer {
            renderer,
            last_frame,
        }
    }

    pub fn attach_text(&mut self, ui : &imgui::Ui, text : &str) { ui.text(text); }

    pub fn update(&mut self, width : f32, height : f32) { self.renderer.update(width, height); }

    pub fn render(
        &mut self,
        _device : &wgpu::Device,
        _queue : &wgpu::Queue,
        _renderer : &mut imgui_wgpu::Renderer,
        ui : &imgui::Ui,
    ) {

        let now = Instant::now();

        self.last_frame = now;

        self.renderer.set_data(_device, _queue, _renderer);

        self.renderer.render(ui);
    }

    pub fn renderer(&self) -> &LayerRenderer { &self.renderer }
}
