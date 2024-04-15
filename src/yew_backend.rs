use crate::Props;
use std::sync::RwLock;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
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
                log::warn!("A");
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    log::warn!("B");
                    let state = RendererState::init_web(canvas).await;
                    log::warn!("C");
                    let mut real_renderer = R::init(&state).await;
                    log::warn!("D");
                    real_renderer.render(&state);
                    log::warn!("E");

                    renderer_state.set(Some(RwLock::new(state)));
                    renderer.set(Some(RwLock::new(real_renderer)));
                    start_render_loop.set(RENDER_LOOP);
                }
            },
            canvas_ref2.clone(),
        );
    }

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
                    .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap();
            }));

            window
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .unwrap();
        },
        start_render_loop.clone(),
    );

    html! {
        <div>
            <h1>{&props.title}</h1>
            <canvas style="width: 100%" ref={canvas_ref} width={WIDTH.to_string()} height={HEIGHT.to_string()}/>
        </div>
    }
}

pub fn start<R: crate::Renderer>(props: Props) {
    yew::Renderer::<App<R>>::with_props(props).render();
}
