use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use glam::{Vec3A};
use wgpu::{ColorTargetState, MultisampleState, Queue, RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor, SurfaceConfiguration};
use wgpu::{Color, CommandEncoder, Device};
use winit::dpi::{PhysicalPosition};
use winit::event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode};
use scenelib::camera::{Camera, CameraNode};
use scenelib::scene::{StaticRenderState, RenderScene, RenderCallState};
use crate::input::{InputHandler};

struct EngineCoreState {
    render_pipeline: wgpu::RenderPipeline,
    triangle_render_bundle: RenderBundle,
    render_scene: RenderScene,
    input_handler: InputHandler,
}

pub struct EngineInstance {
    device: Rc<Device>,
    queue: Rc<Queue>,
    surface_config: Rc<RefCell<SurfaceConfiguration>>,
    color_target_state: ColorTargetState,
    pub multisample_state: MultisampleState,
    engine_core_state: Option<EngineCoreState>,
}

#[derive(Debug, PartialEq)]
pub struct ViewportRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewportRegion {
    pub const ZERO: ViewportRegion = ViewportRegion {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
}

impl EngineInstance {
    pub fn new(device: Rc<Device>, queue: Rc<Queue>, surface_config: Rc<RefCell<SurfaceConfiguration>>) -> EngineInstance {
        let surface_format = surface_config.borrow().format;
        EngineInstance {
            device,
            queue,
            surface_config,
            color_target_state: ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: Default::default(),
            },
            multisample_state: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            engine_core_state: None,
        }
    }

    #[profiling::function]
    pub fn start(&mut self) {
        let triangle_render_bundle;
        {
            let mut triangle_render_bundle_encoder = self.device.create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
                label: Some("TriangleRenderBundleEncoder"),
                color_formats: &[self.surface_config.borrow_mut().format],
                depth_stencil: None,
                sample_count: self.multisample_state.count,
                multiview: None,
            });
            triangle_render_bundle_encoder.draw(0..3, 0..1);
            triangle_render_bundle = triangle_render_bundle_encoder.finish(&RenderBundleDescriptor {
                label: Some("TriangleRenderBundle"),
            });
        }

        let mut render_scene = RenderScene::new(StaticRenderState {
            device: self.device.clone(),
            queue: self.queue.clone(),
            bind_group_layouts: Vec::new(),
        });

        CameraNode::add_new(
            0,
            Camera::new(
                Vec3A::new(0.0, 0.0, -5.0),
                Vec3A::new(0.0, 0.0, 1.0),
                70.0, 0.01, 1000.0, 1.0,
                Vec3A::new(0.0, 1.0, 0.0),
            ),
            &mut render_scene,
        );

        let shader = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../cres/shaders/shader.frag.wgsl"))),
        });

        let mut layouts = Vec::new();

        for layout in &render_scene.static_render_state.bind_group_layouts {
            layouts.push(layout);
        }

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[self.color_target_state.clone()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: self.multisample_state,
            multiview: None,
        });
        self.engine_core_state = Some(EngineCoreState { render_pipeline, triangle_render_bundle, render_scene, input_handler: InputHandler::new() });
    }

    #[profiling::function]
    pub fn render<'a, 'b: 'a>(&'b mut self, command_encoder: &'a mut CommandEncoder, surface_texture_view: &wgpu::TextureView, mutisampled_framebuffer: Option<&wgpu::TextureView>, viewport_region: &ViewportRegion, delta_time: f64) {
        if viewport_region == &ViewportRegion::ZERO || self.engine_core_state.is_none() {
            return;
        }

        let engine_state: &mut EngineCoreState = self.engine_core_state.as_mut().unwrap();

        // Pre-rendering
        {
            // Hacky camera controller
            // TODO: make this an ECS component when we have ECS
            {
                let mut speed = 2.0f32;

                let handler_option = &mut engine_state.input_handler.get_primary();

                match handler_option {
                    Some(handler) => {
                        let camera_node: &mut CameraNode = engine_state.render_scene.get_node_by_id(0).unwrap();

                        let mut move_forward = 0.0f32;
                        let mut move_strafing = 0.0f32;
                        let mut move_up = 0.0f32;

                        if handler.is_key_pressed(VirtualKeyCode::LControl) {
                            speed = 8.0f32;
                        }

                        // Get move_foward and move_strafing from input handler
                        {
                            if handler.is_key_pressed(VirtualKeyCode::W) {
                                move_forward = 1.0f32;
                            }
                            if handler.is_key_pressed(VirtualKeyCode::S) {
                                move_forward = -1.0f32;
                            }
                            if handler.is_key_pressed(VirtualKeyCode::A) {
                                move_strafing = -1.0f32;
                            }
                            if handler.is_key_pressed(VirtualKeyCode::D) {
                                move_strafing = 1.0f32;
                            }
                            if handler.is_key_pressed(VirtualKeyCode::Space) {
                                move_up = 1.0f32;
                            }
                            if handler.is_key_pressed(VirtualKeyCode::LShift) {
                                move_up = -1.0f32;
                            }
                        }
                        if move_forward != 0.0f32 {
                            camera_node.set_position(camera_node.position() + camera_node.direction() * (move_forward * speed * delta_time as f32));
                        }
                        if move_strafing != 0.0f32 {
                            camera_node.set_position(camera_node.position() + camera_node.right() * (move_strafing * speed * delta_time as f32));
                        }
                        if move_up != 0.0f32 {
                            camera_node.set_position(camera_node.position() + camera_node.up() * (move_up * speed * delta_time as f32));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Begin rendering
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("MainEngineRenderPass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: if self.multisample_state.count == 1 { &surface_texture_view } else { mutisampled_framebuffer.unwrap() },
                    resolve_target: if self.multisample_state.count == 1 { None } else { Some(&surface_texture_view) },
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color::TRANSPARENT),
                        // Storing pre-resolve MSAA data is unnecessary if it isn't used later.
                        // On tile-based GPU, avoid store can reduce your app's memory footprint.
                        store: if self.multisample_state.count == 1 { true } else { false },
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_viewport(viewport_region.x, viewport_region.y, viewport_region.width, viewport_region.height, 0.0, 1.0);
            render_pass.set_pipeline(&engine_state.render_pipeline);

            engine_state.render_scene.render(&mut RenderCallState { render_pass: &mut render_pass });

            render_pass.execute_bundles(std::iter::once(&engine_state.triangle_render_bundle));
        }
    }

    #[profiling::function]
    pub fn resize(&mut self, viewport_region: &ViewportRegion) {
        if viewport_region == &ViewportRegion::ZERO || self.engine_core_state.is_none() {
            return;
        }

        let engine_state: &mut EngineCoreState = self.engine_core_state.as_mut().unwrap();
        {
            let camera_node: &mut CameraNode = engine_state.render_scene.get_node_by_id(0).unwrap();
            camera_node.set_aspect(viewport_region.width as f32 / viewport_region.height as f32);
        }
    }

    #[profiling::function]
    pub fn handle_key_state(&mut self, device_id: DeviceId, key_code: VirtualKeyCode, key_state: ElementState, _is_synthetic: bool, _delta_time: f64) {
        let engine_state: &mut EngineCoreState = self.engine_core_state.as_mut().unwrap();
        let input_handler = &mut engine_state.input_handler;
        input_handler.set_key_pressed(device_id, key_code, key_state);
    }

    #[profiling::function]
    pub fn handle_mouse_button_event(&mut self, _device_id: DeviceId, _mouse_button: MouseButton, _button_state: ElementState, _delta_time: f64) {}

    #[profiling::function]
    pub fn handle_mouse_wheel(&mut self, _device_id: DeviceId, _delta: MouseScrollDelta, _phase: TouchPhase, _delta_time: f64) {}

    #[profiling::function]
    pub fn handle_mouse_move(&mut self, _device_id: DeviceId, _mouse_position: PhysicalPosition<f64>, _delta_time: f64) {}

    pub fn should_grab_cursor(&self) -> bool {
        return true;
    }
}