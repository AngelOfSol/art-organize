use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use imgui_winit_support::WinitPlatform;
use std::time::Instant;
use wgpu::Extent3d;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{app::App, raw_image::RawImage, style::modify_style};

pub struct GuiContext {
    _instance: wgpu::Instance,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
    window: Window,
    device: wgpu::Device,
    imgui: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    last_frame: Instant,
    last_cursor: Option<Option<MouseCursor>>,
}

pub fn run_event_loop(event_loop: EventLoop<()>, mut context: GuiContext, mut app: App) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        app.update(&mut context);

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let size = context.window.inner_size();
                if size.width != 0 && size.height != 0 {
                    let sc_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        width: size.width as u32,
                        height: size.height as u32,
                        present_mode: wgpu::PresentMode::Mailbox,
                    };

                    context.swap_chain =
                        context.device.create_swap_chain(&context.surface, &sc_desc);
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                context.window.request_redraw();
            }
            Event::RedrawEventsCleared => {
                context.render(&mut app);
            }
            _ => (),
        }

        context
            .platform
            .handle_event(context.imgui.io_mut(), &context.window, &event);
    });
}

impl GuiContext {
    pub async fn create(event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let (window, size, surface) = {
            let window = Window::new(event_loop).unwrap();
            window.set_inner_size(LogicalSize {
                width: 1280.0,
                height: 720.0,
            });
            window.set_title("ArtOrganize");
            let size = window.inner_size();

            let surface = unsafe { instance.create_surface(&window) };

            (window, size, surface)
        };

        let hidpi_factor = window.scale_factor();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        // Set up swap chain
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // Set up dear imgui
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        modify_style(imgui.style_mut());

        let renderer_config = RendererConfig {
            texture_format: sc_desc.format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

        Ok(GuiContext {
            _instance: instance,
            window,
            surface,
            device,
            platform,
            swap_chain,
            imgui,
            renderer,
            queue,
            last_frame: Instant::now(),
            last_cursor: None,
        })
    }

    pub fn render(&mut self, app: &mut App) {
        let now = Instant::now();
        self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;

        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                //eprintln!("dropped frame: {:?}", e);
                return;
            }
        };
        self.platform
            .prepare_frame(self.imgui.io_mut(), &self.window)
            .expect("Failed to prepare frame");
        let ui = self.imgui.frame();

        app.render(&ui, self.window.inner_size().cast());

        let mut encoder: wgpu::CommandEncoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if self.last_cursor != Some(ui.mouse_cursor()) {
            self.last_cursor = Some(ui.mouse_cursor());
            self.platform.prepare_render(&ui, &self.window);
        }

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.007,
                        g: 0.007,
                        b: 0.007,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.renderer
            .render(ui.render(), &self.queue, &self.device, &mut rpass)
            .expect("Rendering failed");

        drop(rpass);

        self.queue.submit(Some(encoder.finish()));
    }

    pub fn load(&mut self, raw: &RawImage) -> TextureId {
        let texture_config = TextureConfig {
            size: Extent3d {
                width: raw.width,
                height: raw.height,
                ..Default::default()
            },

            label: None,
            ..Default::default()
        };

        let texture = Texture::new(&self.device, &self.renderer, texture_config);

        texture.write(&self.queue, &raw.data, raw.width, raw.height);
        self.renderer.textures.insert(texture)
    }
}

pub struct ImageIds {
    base: TextureId,
    thumbnail: TextureId,
}
