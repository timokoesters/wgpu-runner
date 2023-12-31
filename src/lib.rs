use std::collections::BTreeSet;

pub use winit::event;

use glam::Vec2;
use instant::Instant;
use winit::{
    event::{DeviceEvent, VirtualKeyCode, WindowEvent},
    window::Window,
};
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
pub mod yew_backend;

pub mod winit_backend;

pub trait Renderer: 'static + Sized {
    fn init(state: &RendererState) -> Self;
    fn on_window_event(&mut self, state: &RendererState, event: &WindowEvent);
    fn on_device_event(&mut self, state: &RendererState, event: &DeviceEvent);
    fn on_resize(&mut self, state: &RendererState);
    fn render(&mut self, state: &RendererState);
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub title: String,
}
impl Default for Props {
    fn default() -> Self {
        Self {
            title: "Default title!".to_owned(),
        }
    }
}

pub struct CursorState {
    pub position: Vec2,
    pub dragging_from: Option<Vec2>,
}

pub struct RendererState {
    pub width: u32,
    pub height: u32,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub pressed_keys: BTreeSet<VirtualKeyCode>,
    pub cursor: CursorState,
    pub queue: wgpu::Queue,
    pub start: Instant,
}

impl RendererState {
    #[cfg(target_arch = "wasm32")]
    fn init_web(canvas: web_sys::HtmlCanvasElement) -> Self {
        let width = canvas.width();
        let height = canvas.height();

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface_from_canvas(canvas).unwrap();
        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap()
        });
        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits {
                            ..wgpu::Limits::downlevel_webgl2_defaults()
                        },
                        label: None,
                    },
                    None,
                )
                .await
                .unwrap()
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        RendererState {
            width,
            height,
            device,
            surface,
            config,
            queue,
            pressed_keys: BTreeSet::new(),
            cursor: CursorState {
                position: Vec2::ZERO,
                dragging_from: None,
            },
            start: Instant::now(),
        }
    }

    fn init_winit(window: &Window) -> Self {
        let instance = wgpu::Instance::default();

        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap()
        });
        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits {
                            max_uniform_buffer_binding_size: 32000000,
                            max_storage_buffer_binding_size: 128 << 21,
                            ..wgpu::Limits::downlevel_defaults()
                        },
                        label: None,
                    },
                    None,
                )
                .await
                .unwrap()
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        RendererState {
            width,
            height,
            device,
            surface,
            config,
            queue,
            pressed_keys: BTreeSet::new(),
            cursor: CursorState {
                position: Vec2::ZERO,
                dragging_from: None,
            },
            start: Instant::now(),
        }
    }
}

pub fn start<R: Renderer>(props: Props) {
    #[cfg(target_arch = "wasm32")]
    yew_backend::start::<R>(props);

    #[cfg(not(target_arch = "wasm32"))]
    winit_backend::start::<R>(props);
}
