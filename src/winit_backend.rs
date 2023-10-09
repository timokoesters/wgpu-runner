use crate::Props;
use crate::RendererState;

use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;

pub fn start<R: crate::Renderer>(props: Props) {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_title(&props.title)
        .build(&event_loop)
        .unwrap();

    let renderer = R::init(RendererState::init_winit(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
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
            _ => {}
        },
        Event::RedrawRequested(_) => {
            renderer.render();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
