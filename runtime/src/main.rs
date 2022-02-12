use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use dyngine_core::engine::{EngineInstance, ViewportRegion};

async fn run(event_loop: EventLoop<()>, window: Window) {
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

    event_loop.run(move |event, _, control_flow| {
        // event_loop.run never returns, therefore we must take ownership of the resources
        // to ensure the resources are properly cleaned up.
        let _ = (&instance, &adapter, &engine_instance);
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
                    let viewport_region = ViewportRegion {
                        x: 0.0,
                        y: 0.0,
                        width: surface_config.width as f32,
                        height: surface_config.height as f32,
                    };
                    engine_instance.render(&mut command_encoder, &viewport_view, &viewport_region);
                    queue.submit(Some(command_encoder.finish()));
                }
                output_frame.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}


fn main() {
    let event_loop = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_title("Dyngine")
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_min_inner_size(LogicalSize { width: 720, height: 480 })
        .build(&event_loop)
        .unwrap();

    {
        // Temporarily avoid srgb formats for the swap chain on the web
        pollster::block_on(run(event_loop, window));
    }
}
