use crate::layer_renderer::LayerRenderer;
use imgui::*;
use std::time::Instant;

use crate::State;

use crate::Camera;

#[repr(C)]

pub struct Vector2<T> {
    pub x : T,
    pub y : T,
}

pub struct Layer {
    pub renderer : LayerRenderer,
    pub last_frame : Instant,
}

impl Layer {
    pub fn new(device : &wgpu::Device, queue : &wgpu::Queue, bytes : &[u8], path : &str) -> Layer {

        let renderer = LayerRenderer::new(device, queue, bytes, path);

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

    pub fn render(&mut self, device : &wgpu::Device, queue : &wgpu::Queue) {

        let now = Instant::now();

        self.last_frame = now;

        self.renderer.render(device, queue);
    }
}
