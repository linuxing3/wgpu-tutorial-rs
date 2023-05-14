#[derive(Debug)]

pub struct CommandBufferController {}

impl CommandBufferController {
    pub fn create_render_pass<'a>(
        // _device : &wgpu::Device,
        // _queue : &wgpu::Queue,
        // _renderer : &imgui_wgpu::Renderer,
        // _imgui_frame : &imgui::Ui,
        view : &'a wgpu::TextureView,
        command_encoder : &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {

        let renderpass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label : None,
            color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                view : &view,
                resolve_target : None,
                ops : wgpu::Operations {
                    load : wgpu::LoadOp::Load, // Do not clear
                    // load: wgpu::LoadOp::Clear(clear_color),
                    store : true,
                },
            })],
            depth_stencil_attachment : None,
        });

        renderpass
    }
}
