use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::time::{Duration, Instant};
use wgpu::SurfaceConfiguration;
use winit::dpi::{LogicalSize};
use winit::event::{DeviceEvent, Event, WindowEvent};
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
        queue = Rc::new(q);
    }

    let surface_format = surface.get_preferred_format(&adapter).unwrap();

    let surface_config: Rc<RefCell<SurfaceConfiguration>> = Rc::new(RefCell::new(wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    }));

    let mut engine_instance = EngineInstance::new(device.clone(), queue.clone(), surface_config.clone());

    engine_instance.start();
    surface.configure(&device, surface_config.borrow_mut().deref());

    let mut last_frame_end = Instant::now();
    let mut last_frame_time = Duration::from_secs(0);

    let mut frame = 0i64;

    window.set_visible(true); // engine startup complete

    let mut grabbed_cursor = false;

    let mut window_has_focus = false;
    let mut last_focused_time = Instant::now();
    let mut last_window_moved = Instant::now();

    let mut mouse_input_valid = false;

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

                        // Resize surface to match window
                        {
                            surface_config_mut.width = size.width;
                            surface_config_mut.height = size.height;
                            surface.configure(&device, surface_config_mut.deref());
                        }

                        // Resize engine instance
                        {
                            let scale_factor = window.scale_factor() as f32;
                            let scaled_viewport_region = ViewportRegion {
                                x: 0.0,
                                y: 0.0,
                                width: surface_config_mut.width as f32 * scale_factor,
                                height: surface_config_mut.height as f32 * scale_factor,
                            };
                            engine_instance.resize(&scaled_viewport_region);
                        }
                    }
                }
                WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                    match input.virtual_keycode {
                        Some(key_code) => {
                            if engine_instance.window_state.has_focus() {
                                engine_instance.handle_key_state(device_id, key_code, input.state, is_synthetic, last_frame_time.as_secs_f64());
                            }
                        }
                        None => {}
                    }
                }
                WindowEvent::MouseInput { device_id, button, state, .. } => {
                    if engine_instance.window_state.has_focus() {
                        engine_instance.handle_mouse_button_event(device_id, button, state, last_frame_time.as_secs_f64());
                    }
                }
                WindowEvent::MouseWheel { device_id, delta, phase, .. } => {
                    if engine_instance.window_state.has_focus() {
                        engine_instance.handle_mouse_wheel(device_id, delta, phase, last_frame_time.as_secs_f64());
                    }
                }
                WindowEvent::Focused(focused) => {
                    window_has_focus = focused;
                    engine_instance.window_state.set_focus(focused);
                    last_focused_time = Instant::now();
                }
                WindowEvent::Moved(_) => {
                    last_window_moved = Instant::now();
                    mouse_input_valid = false;
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
            Event::DeviceEvent {
                event,
                device_id,
            } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    if !mouse_input_valid {
                        return;
                    }
                    if engine_instance.window_state.has_focus() {
                        engine_instance.handle_mouse_motion(device_id, delta, last_frame_time.as_secs_f64());
                    }
                }
                _ => {}
            },
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

                // Prevent mouse jumping when window is focused/dragged
                if !mouse_input_valid && last_window_moved.elapsed().as_millis().min(last_focused_time.elapsed().as_millis()) >= 50 {
                    mouse_input_valid = true;
                }

                if mouse_input_valid {
                    // Grab cursor, if engine requests it
                    if engine_instance.should_grab_cursor() && window_has_focus {
                        if !grabbed_cursor {
                            window.set_cursor_grab(true).unwrap();
                            window.set_cursor_visible(false);
                            grabbed_cursor = true;
                        }
                        // window.set_cursor_position(PhysicalPosition::new(surface_config.borrow().width / 2, surface_config.borrow().height / 2)).unwrap();
                    } else {
                        if grabbed_cursor {
                            window.set_cursor_grab(false).unwrap();
                            window.set_cursor_visible(true);
                            grabbed_cursor = false;
                        }
                    }
                }

                // Engine render
                {
                    let viewport_view = output_frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut command_encoder = device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor { label: Some("EngineRender") }
                    );

                    let surface_config_mut = surface_config.borrow_mut();

                    // No scale factor needed to render correctly
                    let viewport_region = ViewportRegion {
                        x: 0.0,
                        y: 0.0,
                        width: surface_config_mut.width as f32,
                        height: surface_config_mut.height as f32,
                    };

                    let primary_camera = engine_instance.engine_core_state.as_ref().unwrap().ecs_world.get_primary_camera().unwrap();
                    let delta_time = last_frame_time.as_secs_f64();

                    engine_instance.render(&mut command_encoder, &viewport_view, None, &viewport_region, primary_camera, delta_time);
                    queue.submit(Some(command_encoder.finish()));
                }

                output_frame.present();

                if !last_frame_time.is_zero() && frame % 600 == 0 {
                    let fps = (1.0 / last_frame_time.as_secs_f32()) as i32;
                    let fps_str = format!("Dyngine ({} FPS)", fps);
                    window.set_title(&fps_str);
                }
                let now = Instant::now();
                last_frame_time = now.duration_since(last_frame_end);
                last_frame_end = now;

                profiling::finish_frame!();
                frame += 1;
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
    print!("Giving profiler time to attach");
    use std::thread::sleep;
    use std::io;
    use std::io::Write;
    for _ in 0..100 {
        print!(".");
        profiling::scope!("Wait for Optick...");
        sleep(Duration::from_millis(100));
        profiling::finish_frame!();
        match io::stdout().flush() {
            Ok(_) => {}
            Err(_) => {}
        }
    }
    println!();
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
        .with_min_inner_size(LogicalSize { width: 1280, height: 720 })
        .with_visible(false)
        .build(&event_loop)
        .unwrap();
    {
        pollster::block_on(run(event_loop, window));
    }
}
