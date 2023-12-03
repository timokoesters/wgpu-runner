use crate::Props;
use crate::Renderer;
use crate::RendererState;

use glam::UVec2;
use glam::Vec2;
use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn start<R>(props: Props)
where
    R: Renderer,
{
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(&props.title)
        .build(&event_loop)
        .unwrap();

    let mut state = RendererState::init_winit(&window);
    let mut app = R::init(&state);

    app.on_resize(&state);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, window_id } if window_id == window.id() => {
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
                WindowEvent::Focused(false) => {
                    state.pressed_keys.clear();
                }
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        state.width = new_size.width;
                        state.height = new_size.height;
                        state.config.width = new_size.width;
                        state.config.height = new_size.height;
                        state.surface.configure(&state.device, &state.config);
                        app.on_resize(&state);
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    state.cursor.position = Vec2::new(
                        position.x as f32 / state.width as f32 * 2.0 - 1.0,
                        -position.y as f32 / state.height as f32 * 2.0 + 1.0,
                    );
                }
                WindowEvent::MouseInput {
                    state: button_state,
                    button: MouseButton::Left | MouseButton::Middle | MouseButton::Right,
                    ..
                } => {
                    if button_state == ElementState::Pressed {
                        if state.cursor.dragging_from == None {
                            state.cursor.dragging_from = Some(state.cursor.position);
                        }
                    } else {
                        state.cursor.dragging_from = None;
                    }
                }
                _ => {}
            };
            app.on_window_event(&state, &event);
        }
        Event::DeviceEvent { event, .. } => {
            app.on_device_event(&state, &event);
        }
        Event::RedrawRequested(_) => {
            app.render(&state);
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
