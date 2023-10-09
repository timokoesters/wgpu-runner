use crate::Props;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

use crate::RendererState;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const RENDER_LOOP: bool = true;

#[function_component]
fn App<R: crate::Renderer>(props: &Props) -> Html {
    let canvas_ref = use_node_ref();
    let start_render_loop = use_state(|| false);
    let renderer = use_state(|| None);
    use_effect_with_deps(
        {
            // Initialize renderer for canvas
            let renderer = renderer.clone();
            let start_render_loop = start_render_loop.clone();
            move |canvas_ref: &NodeRef| {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let real_renderer = R::init(RendererState::init_web(canvas));
                    real_renderer.render();
                    renderer.set(Some(real_renderer));
                    start_render_loop.set(RENDER_LOOP);
                }
                || ()
            }
        },
        canvas_ref.clone(),
    );

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
                renderer.as_ref().unwrap().render();
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
