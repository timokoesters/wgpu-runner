use instant::Instant;
use winit::window::Window;
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
pub mod yew_backend;

pub mod winit_backend;

pub trait Renderer: 'static + Sized {
    fn init(state: RendererState) -> Self;
    fn render(&self);
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

pub struct RendererState {
    pub width: u32,
    pub height: u32,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
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
