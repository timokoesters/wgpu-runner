use crate::Props;
use std::sync::RwLock;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlCanvasElement, HtmlElement, MouseEvent};
use winit::event::DeviceEvent;
use winit::keyboard::{KeyCode, PhysicalKey};
use yew::prelude::*;
use yew::suspense::use_future_with_deps;

use crate::RendererState;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const RENDER_LOOP: bool = true;

#[function_component]
fn App<R: crate::Renderer>(props: &Props) -> Html {
    let canvas_ref = use_node_ref();
    let start_render_loop = use_state(|| false);
    let renderer_state = use_state(|| None);
    let renderer = use_state(|| None);
    {
        let renderer = renderer.clone();
        let renderer_state = renderer_state.clone();
        let start_render_loop = start_render_loop.clone();
        let canvas_ref = canvas_ref.clone();
        let canvas_ref2 = canvas_ref.clone();
        use_future_with_deps(
            |_| async move {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let state = RendererState::init_web(canvas).await;
                    let mut real_renderer = R::init(&state).await;
                    real_renderer.render(&state);

                    renderer_state.set(Some(RwLock::new(state)));
                    renderer.set(Some(RwLock::new(real_renderer)));
                    start_render_loop.set(RENDER_LOOP);
                }
            },
            canvas_ref2.clone(),
        );
    }

    {
        let renderer = renderer.clone();
        let renderer_state = renderer_state.clone();
        use_effect_with_deps(
            move |start_render_loop| {
                if !**start_render_loop {
                    return;
                }
                let window = web_sys::window().unwrap();
                let window2 = window.clone();
                let f = Rc::new(RefCell::<Option<Closure<dyn FnMut()>>>::new(None));
                let g = f.clone();
                *g.borrow_mut() = Some(Closure::new(move || {
                    renderer
                        .as_ref()
                        .unwrap()
                        .write()
                        .unwrap()
                        .render(&renderer_state.as_ref().unwrap().read().unwrap());
                    window2
                        .request_animation_frame(
                            f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                        )
                        .unwrap();
                }));

                window
                    .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap();
            },
            start_render_loop.clone(),
        );
    }

    let mousemove = {
        let renderer = renderer.clone();
        let renderer_state = renderer_state.clone();
        Callback::from(move |e: MouseEvent| {
            let element = e.target().unwrap().dyn_into::<Element>().unwrap();
            if gloo::utils::document().pointer_lock_element() != Some(element) {
                return;
            }
            let Some(state) = &renderer_state.as_ref().map(|s| s.read().unwrap()) else {
                return;
            };
            renderer.as_ref().unwrap().write().unwrap().on_device_event(
                &state,
                &DeviceEvent::MouseMotion {
                    delta: (2.0 * e.movement_x() as f64, 2.0 * e.movement_y() as f64),
                },
            );
        })
    };

    let keydown = {
        let renderer_state = renderer_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            let Some(state) = &mut renderer_state.as_ref().map(|s| s.write().unwrap()) else {
                return;
            };
            gloo::console::warn!("HAA down");
            if let Some(keycode) = key_to_keycode(&e.key()) {
                state.pressed_keys.insert(PhysicalKey::Code(keycode));
            }
        })
    };

    let keyup = {
        let renderer_state = renderer_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            let Some(state) = &mut renderer_state.as_ref().map(|s| s.write().unwrap()) else {
                return;
            };
            gloo::console::warn!("HAA up");
            if let Some(keycode) = key_to_keycode(&e.key()) {
                state.pressed_keys.remove(&PhysicalKey::Code(keycode));
            }
        })
    };

    use_effect(move || {
        Box::leak(Box::new(gloo::events::EventListener::new(
            &web_sys::window().unwrap(),
            "resize",
            move |e| {
                gloo::console::warn!("resize");
                let canvas = gloo::utils::document()
                    .get_element_by_id("main-canvas")
                    .unwrap();
                gloo::console::warn!("resize 2");
                let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
                gloo::console::warn!("resize 3");
                let Some(state) = &mut renderer_state.as_ref().map(|s| s.write().unwrap()) else {
                    gloo::console::warn!("reszize 4");
                    return;
                };
                gloo::console::warn!("RESIZE", canvas.client_width(), canvas.client_height());
                gloo::console::warn!("RESIZE", canvas.width(), canvas.height());

                let width = canvas.client_width() as u32;
                let height = canvas.client_height() as u32;
                canvas.set_width(width);
                canvas.set_height(height);
                state.width = width;
                state.height = height;
                state.config.width = width;
                state.config.height = height;
                state.surface.configure(&state.device, &state.config);

                renderer
                    .as_ref()
                    .unwrap()
                    .write()
                    .unwrap()
                    .on_resize(&state);
            },
        )));
    });

    html! {
        <div style="text-align: center;">
            <h1>{&props.title}</h1>
            <canvas
                id="main-canvas"
                tabindex="-1"
                style=""
                ref={canvas_ref}
                onclick={Callback::from(|e: MouseEvent| {
                    gloo::console::warn!("HIT");
                    let element = e.target().unwrap().dyn_into::<HtmlElement>().unwrap();
                    element.focus();
                    element.request_pointer_lock();
                })}
                onmousemove={mousemove}
                onkeydown={keydown}
                onkeyup={keyup}
            />
        </div>
    }
}

pub fn start<R: crate::Renderer>(props: Props) {
    yew::Renderer::<App<R>>::with_props(props).render();
}

fn key_to_keycode(key: &str) -> Option<KeyCode> {
    let key = match &*key.to_lowercase() {
        "shift" => KeyCode::ShiftLeft,
        "w" => KeyCode::KeyW,
        "a" => KeyCode::KeyA,
        "s" => KeyCode::KeyS,
        "d" => KeyCode::KeyD,
        k => {
            gloo::console::warn!("Unknown key: ", k);
            return None;
        }
    };
    Some(key)
}
