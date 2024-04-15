use std::sync::Arc;

use crate::Props;
use crate::Renderer;
use crate::RendererState;

use glam::UVec2;
use glam::Vec2;
use winit::dpi::PhysicalSize;
use winit::keyboard::Key;
use winit::keyboard::NamedKey;
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

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title(&props.title)
            .build(&event_loop)
            .unwrap(),
    );
    if props.capture_cursor {
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .unwrap();
        window.set_cursor_visible(false);
    }

    let (mut state, mut app) = pollster::block_on(async {
        let state = RendererState::init_winit(window.clone()).await;
        let app = R::init(&state).await;
        (state, app)
    });

    app.on_resize(&state);

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, window_id } => {
                if let WindowEvent::KeyboardInput { event, .. } = &event {
                    if event.state.is_pressed() {
                        state.pressed_keys.insert(event.physical_key);
                    } else {
                        state.pressed_keys.remove(&event.physical_key);
                    }
                }

                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => elwt.exit(),
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
                    WindowEvent::RedrawRequested => {
                        app.render(&state);
                        window.request_redraw();
                    }
                    _ => {}
                };
                app.on_window_event(&state, &event);
            }
            Event::DeviceEvent { event, .. } => {
                app.on_device_event(&state, &event);
            }
            _ => {}
        })
        .unwrap();
}
