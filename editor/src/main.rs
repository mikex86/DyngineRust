mod gui;
mod i18n;

use std::cell::RefCell;
use std::iter;
use std::ops::Deref;
use std::rc::Rc;
use std::time::{Duration, Instant};
use egui::{Color32, FontDefinitions, Style, TextStyle, Visuals};
use egui::style::{Widgets, WidgetVisuals};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::App;
use wgpu::{Device, SurfaceConfiguration, TextureFormat, TextureView};

use winit::{
    event_loop::{EventLoop},
    window::Window,
};

use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event::Event::UserEvent;
use winit::event_loop::ControlFlow;
use winit::window::{WindowBuilder};
use dyngine_core::engine::{EngineInstance, ViewportRegion};
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

    let (device, queue);
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

    let surface_config: Rc<RefCell<SurfaceConfiguration>> = Rc::new(RefCell::new(wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    }));

    let mut engine_instance = EngineInstance::new(device.clone(), surface_config.clone());

    fn create_multi_sampled_frame_buffer(device: &Device, size: &PhysicalSize<u32>, multi_sample_count: u32, surface_format: TextureFormat) -> TextureView {
        return device
            .create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: multi_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: surface_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: None,
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    let mut multisampled_frame_buffer = create_multi_sampled_frame_buffer(&device, &size, engine_instance.multisample_state.count, surface_format);

    engine_instance.start();

    surface.configure(&device, surface_config.borrow_mut().deref());

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
                        // Resize surface
                        {
                            let mut surface_config_mut = surface_config.borrow_mut();
                            surface_config_mut.width = size.width;
                            surface_config_mut.height = size.height;
                        }
                        surface.configure(&device, surface_config.borrow_mut().deref());

                        // Resize multi sampled frame buffer
                        {
                            multisampled_frame_buffer = create_multi_sampled_frame_buffer(&device, &size, engine_instance.multisample_state.count, surface_format);
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
            Event::RedrawRequested(..) => {
                profiling::scope!("RedrawRequested");

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
                        &wgpu::CommandEncoderDescriptor { label: Some("MainEngineCommandEncoder") }
                    );
                    let viewport_region = &egui_app.viewport_region;
                    let scale_factor = window.scale_factor() as f32;
                    let scaled_viewport_region = ViewportRegion {
                        x: viewport_region.x * scale_factor,
                        y: viewport_region.y * scale_factor,
                        width: viewport_region.width * scale_factor,
                        height: viewport_region.height * scale_factor
                    };
                    engine_instance.render(&mut command_encoder, &viewport_view, Some(&multisampled_frame_buffer), &scaled_viewport_region);
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
                        &wgpu::CommandEncoderDescriptor { label: Some("EguiRender") }
                    );

                    let surface_config_mut = surface_config.borrow_mut();
                    let screen_descriptor = ScreenDescriptor {
                        physical_width: surface_config_mut.width,
                        physical_height: surface_config_mut.height,
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

                profiling::finish_frame!();
            }
            Event::MainEventsCleared | UserEvent(ExampleEvent::RequestRedraw) => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

#[cfg(feature = "profile-with-optick")]
fn wait_for_profiler() {
    use std::thread::sleep;
    for _ in 0..100 {
        profiling::scope!("Wait for Optick...");
        sleep(Duration::from_millis(100));
        profiling::finish_frame!();
    }
}

#[profiling::function]
fn main() {
    profiling::register_thread!("Engine");

    #[cfg(feature = "profile-with-optick")]
        wait_for_profiler();

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
        pollster::block_on(run(event_loop, window));
    }
}
