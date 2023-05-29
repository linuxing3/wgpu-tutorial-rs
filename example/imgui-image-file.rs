extern crate imgui_winit_support;
extern crate wgpu_tutorial_rs;

use wgpu_tutorial_rs::state::State;

use pollster::block_on;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// run entry
pub async fn run() {

    env_logger::init();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {

        match event {
            Event::RedrawEventsCleared => {

                state.update();

                // render entry
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {

                state.update();
            }
            Event::MainEventsCleared => {

                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            //
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {

                state.resize(state.window().inner_size());
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {

                if !state.input(event) {

                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {

                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {

                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        state
            .platform
            .handle_event(state.imgui_context.io_mut(), &state.window, &event);
    });
}

fn main() { block_on(run()); }
