use glam::Vec2;
use gloo::net::http::Request;
use instant::Instant;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::PathBuf;
use std::sync::Arc;
use winit::keyboard::Key;
use winit::keyboard::PhysicalKey;
use winit::{
    event::{DeviceEvent, WindowEvent},
    keyboard::KeyCode,
    window::Window,
};
use yew::prelude::*;

pub use wgpu;
pub use winit::event;
pub use winit::keyboard;

#[cfg(target_arch = "wasm32")]
pub mod yew_backend;

#[cfg(not(target_arch = "wasm32"))]
pub mod winit_backend;

pub trait Renderer: 'static + Sized {
    fn init(state: &RendererState) -> impl std::future::Future<Output = Self>;
    fn on_window_event(&mut self, state: &RendererState, event: &WindowEvent);
    fn on_device_event(&mut self, state: &RendererState, event: &DeviceEvent);
    fn on_resize(&mut self, state: &RendererState);
    fn render(&mut self, state: &RendererState);
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub title: String,
    pub capture_cursor: bool,
}
impl Default for Props {
    fn default() -> Self {
        Self {
            title: "Default title!".to_owned(),
            capture_cursor: false,
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
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub pressed_keys: BTreeSet<PhysicalKey>,
    pub cursor: CursorState,
    pub queue: wgpu::Queue,
    pub start: Instant,
}

impl RendererState {
    #[cfg(target_arch = "wasm32")]
    async fn init_web(canvas: web_sys::HtmlCanvasElement) -> Self {
        let width = canvas.width();
        let height = canvas.height();

        let instance = wgpu::Instance::default();
        let surface: wgpu::Surface<'static> = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits {
                        max_uniform_buffer_binding_size: 65536,
                        max_storage_buffer_binding_size: 128 << 21,
                        max_texture_array_layers: 256 * 3,
                        ..wgpu::Limits::downlevel_defaults()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

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
            desired_maximum_frame_latency: 2,
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

    async fn init_winit(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();

        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                    required_limits: wgpu::Limits {
                        max_uniform_buffer_binding_size: 65536,
                        max_storage_buffer_binding_size: 128 << 21,
                        max_texture_array_layers: 256 * 3,
                        ..wgpu::Limits::downlevel_defaults()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

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
            desired_maximum_frame_latency: 2,
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
    {
        console_log::init_with_level(log::Level::Debug);
        println!("Starting wasm!");
        yew_backend::start::<R>(props);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("Starting native!");
        winit_backend::start::<R>(props);
    }

    println!("Done!");
}

pub async fn file_open(rel_path: &str) -> Option<impl Read + Seek> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut full_path = PathBuf::from("dist/assets/");
        full_path.push(&rel_path);
        File::open(full_path).ok()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let path = "assets/".to_owned() + rel_path;
        Request::get(&path)
            .send()
            .await
            .ok()?
            .binary()
            .await
            .ok()
            .filter(|file| !file.starts_with(b"<!doctype html>"))
            .map(Cursor::new)
    }
}
