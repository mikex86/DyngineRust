mod gui;
mod i18n;

use std::iter;
use std::rc::Rc;
use std::time::{Duration, Instant};
use egui::{Color32, FontDefinitions, Style, TextStyle, Visuals};
use egui::style::{Widgets, WidgetVisuals};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::App;

use winit::{
    event_loop::{EventLoop},
    window::Window,
};

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event::Event::UserEvent;
use winit::event_loop::ControlFlow;
use winit::window::{WindowBuilder};
use dyngine_core::engine::EngineInstance;
use crate::gui::TestApp;


/// A custom event type for the winit app.
enum ExampleEvent {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<ExampleEvent>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(ExampleEvent::RequestRedraw).ok();
    }
}

async fn run(event_loop: EventLoop<ExampleEvent>, window: Window) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let device;
    let queue;
    {
        let (d, q) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            ).await
            .expect("Failed to create device");
        device = Rc::new(d);
        queue = q;
    }

    let surface_format = surface.get_preferred_format(&adapter).unwrap();

    let mut engine_instance = EngineInstance::new(device.clone(), surface_format);

    engine_instance.start();

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    surface.configure(&device, &surface_config);

    let repaint_signal = std::sync::Arc::new(ExampleRepaintSignal(std::sync::Mutex::new(
        event_loop.create_proxy(),
    )));

    let widget_visuals = Widgets::default();
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Style {
            body_text_style: TextStyle::Small,
            override_text_style: None,
            wrap: None,
            spacing: Default::default(),
            interaction: Default::default(),
            // use transparent background to not occlude viewport, which is rendered before the UI
            visuals: Visuals {
                dark_mode: true,
                override_text_color: None,
                widgets: Widgets {
                    noninteractive: WidgetVisuals {
                        bg_fill: Color32::TRANSPARENT,
                        bg_stroke: widget_visuals.noninteractive.bg_stroke,
                        corner_radius: widget_visuals.noninteractive.corner_radius,
                        fg_stroke: widget_visuals.noninteractive.fg_stroke,
                        expansion: widget_visuals.noninteractive.expansion,
                    },
                    inactive: WidgetVisuals {
                        bg_fill: widget_visuals.inactive.bg_fill,
                        bg_stroke: widget_visuals.inactive.bg_stroke,
                        corner_radius: widget_visuals.inactive.corner_radius,
                        fg_stroke: widget_visuals.inactive.fg_stroke,
                        expansion: widget_visuals.inactive.expansion,
                    },
                    hovered: WidgetVisuals {
                        bg_fill: widget_visuals.hovered.bg_fill,
                        bg_stroke: widget_visuals.hovered.bg_stroke,
                        corner_radius: widget_visuals.hovered.corner_radius,
                        fg_stroke: widget_visuals.hovered.fg_stroke,
                        expansion: widget_visuals.hovered.expansion,
                    },
                    active: WidgetVisuals {
                        bg_fill: widget_visuals.active.bg_fill,
                        bg_stroke: widget_visuals.active.bg_stroke,
                        corner_radius: widget_visuals.active.corner_radius,
                        fg_stroke: widget_visuals.active.fg_stroke,
                        expansion: widget_visuals.active.expansion,
                    },
                    open: WidgetVisuals {
                        bg_fill: widget_visuals.open.bg_fill,
                        bg_stroke: widget_visuals.open.bg_stroke,
                        corner_radius: widget_visuals.open.corner_radius,
                        fg_stroke: widget_visuals.open.fg_stroke,
                        expansion: widget_visuals.open.expansion,
                    },
                },
                selection: Default::default(),
                hyperlink_color: Default::default(),
                faint_bg_color: Color32::default(),
                extreme_bg_color: Color32::default(),
                code_bg_color: Color32::default(),
                window_corner_radius: 0.0,
                window_shadow: Default::default(),
                popup_shadow: Default::default(),
                resize_corner_size: 0.0,
                text_cursor_width: 0.0,
                text_cursor_preview: false,
                clip_rect_margin: 0.0,
                button_frame: false,
                collapsing_header_frame: false,
            },
            animation_time: 0.1,
            debug: Default::default(),
            explanation_tooltips: false,
        },
    });

    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

    let translator = Rc::new(crate::i18n::init_i18n("en-US".parse().unwrap()).unwrap());
    let mut egui_app = TestApp::new(translator);

    let egui_start_time = Instant::now();
    let mut previous_egui_frame_time = None;

    let mut last_frame_end = Instant::now();
    let mut last_frame_time = Duration::from_secs(0);

    event_loop.run(move |event, _, control_flow| {
        // event_loop.run never returns, therefore we must take ownership of the resources
        // to ensure the resources are properly cleaned up.
        let _ = (&instance, &adapter, &engine_instance);

        platform.handle_event(&event);

        match event {
            Event::WindowEvent {
                event,
                ..
            } => match event {
                WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, &surface_config);
                    }
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
            Event::RedrawRequested(..) => {
                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        return;
                    }
                    Err(e) => {
                        eprintln!("Dropped frame with error: {:?}", e);
                        return;
                    }
                };
                // Engine render
                {
                    let viewport_view = output_frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut command_encoder = device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor { label: None }
                    );
                    let viewport_region = &egui_app.viewport_region;
                    engine_instance.render(&mut command_encoder, &viewport_view, viewport_region);
                    queue.submit(Some(command_encoder.finish()));
                }

                // egui render
                {
                    let output_view = output_frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    platform.update_time(egui_start_time.elapsed().as_secs_f64());

                    let egui_start = Instant::now();
                    platform.begin_frame();

                    let app_output = epi::backend::AppOutput::default();

                    let mut frame = epi::Frame::new(epi::backend::FrameData {
                        info: epi::IntegrationInfo {
                            name: "egpu_test",
                            web_info: None,
                            cpu_usage: previous_egui_frame_time,
                            native_pixels_per_point: Some(window.scale_factor() as _),
                            prefer_dark_mode: None,
                        },
                        output: app_output,
                        repaint_signal: repaint_signal.clone(),
                    });

                    egui_app.frame_time = last_frame_time;
                    egui_app.update(&platform.context(), &mut frame);

                    let (_output, paint_commands) = platform.end_frame(Some(&window));
                    let paint_jobs = platform.context().tessellate(paint_commands);

                    let egui_frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
                    previous_egui_frame_time = Some(egui_frame_time);

                    let mut command_encoder = device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor { label: Some("egui CommandEncoder") }
                    );

                    let screen_descriptor = ScreenDescriptor {
                        physical_width: surface_config.width,
                        physical_height: surface_config.height,
                        scale_factor: window.scale_factor() as f32,
                    };

                    egui_rpass.update_texture(&device, &queue, &platform.context().font_image());
                    egui_rpass.update_user_textures(&device, &queue);
                    egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                    egui_rpass
                        .execute(&mut command_encoder, &output_view, &paint_jobs, &screen_descriptor, None)
                        .unwrap();

                    queue.submit(iter::once(command_encoder.finish()));
                }
                output_frame.present();

                let now = Instant::now();
                last_frame_time = now.duration_since(last_frame_end);
                last_frame_end = now;
            }
            Event::MainEventsCleared | UserEvent(ExampleEvent::RequestRedraw) => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_title("Dyngine Editor")
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_min_inner_size(LogicalSize { width: 1280, height: 720 })
        .build(&event_loop)
        .unwrap();

    {
        // Temporarily avoid srgb formats for the swap chain on the web
        pollster::block_on(run(event_loop, window));
    }
}
