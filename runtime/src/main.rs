use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::time::{Duration, Instant};
use wgpu::SurfaceConfiguration;
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
    let mut queue;
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

    engine_instance.start();
    surface.configure(&device, surface_config.borrow_mut().deref());

    let mut last_frame_end = Instant::now();
    let mut last_frame_time = Duration::from_secs(0);

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
                        let mut surface_config_mut = surface_config.borrow_mut();
                        surface_config_mut.width = size.width;
                        surface_config_mut.height = size.height;
                        surface.configure(&device, surface_config_mut.deref());
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
                        &wgpu::CommandEncoderDescriptor { label: Some("EngineRender") }
                    );

                    let surface_config_mut = surface_config.borrow_mut();
                    let viewport_region = ViewportRegion {
                        x: 0.0,
                        y: 0.0,
                        width: surface_config_mut.width as f32,
                        height: surface_config_mut.height as f32,
                    };
                    engine_instance.render(&mut queue, &mut command_encoder, &viewport_view, None, &viewport_region);
                    queue.submit(Some(command_encoder.finish()));
                }

                output_frame.present();

                if !last_frame_time.is_zero() {
                    println!("FPS: {}", 1.0 / last_frame_time.as_secs_f32());
                }
                let now = Instant::now();
                last_frame_time = now.duration_since(last_frame_end);
                last_frame_end = now;

                profiling::finish_frame!();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}


#[cfg(feature = "profile-with-optick")]
fn wait_for_profiler() {
    use std::thread::sleep;
    use std::time::Duration;
    for _ in 0..100 {
        profiling::scope!("Wait for Optick...");
        sleep(Duration::from_millis(100));
        profiling::finish_frame!();
    }
}

fn main() {
    #[cfg(feature = "profile-with-optick")]
        wait_for_profiler();

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
        pollster::block_on(run(event_loop, window));
    }
}
