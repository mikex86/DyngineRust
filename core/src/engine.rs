use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use glam::{EulerRot, Quat, Vec3A};
use wgpu::{ColorTargetState, MultisampleState, Queue, RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor, SurfaceConfiguration};
use wgpu::{Color, CommandEncoder, Device};
use winit::event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode};
use scenelib::camera::{CameraRenderNode};
use scenelib::ecs::{CameraEntity, ECSEntityHandle, ECSWorld, MovementInput};
use scenelib::scene::{StaticRenderState, RenderScene, RenderCallState, RenderNodeHandle};
use crate::input::{InputHandler};

pub struct EngineCoreState {
    render_pipeline: wgpu::RenderPipeline,
    triangle_render_bundle: RenderBundle,
    render_scene: RenderScene,
    pub ecs_world: ECSWorld,
    input_handler: InputHandler,
}

impl EngineCoreState {
    pub(crate) fn get_render_node_handle_by_ecs_handle(&self, entity_handle: &ECSEntityHandle) -> Option<&RenderNodeHandle> {
        return self.ecs_world.get_entity(entity_handle)
            .map(|entity| entity.get_render_node())
            .flatten();
    }
}

pub struct WindowState {
    /// Whether the engine currently has focus.
    /// When launched in edtior mode, this indicates whether the editor has focus.
    has_focus: bool,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            has_focus: false
        }
    }

    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    pub fn set_focus(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
    }
}

pub struct EngineInstance {
    device: Rc<Device>,
    queue: Rc<Queue>,
    pub window_state: WindowState,
    surface_config: Rc<RefCell<SurfaceConfiguration>>,
    color_target_state: ColorTargetState,
    pub multisample_state: MultisampleState,
    pub engine_core_state: Option<EngineCoreState>,
    movement_input: MovementInput,
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
            window_state: WindowState::new(),
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
            movement_input: MovementInput::new(),
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

        let mut ecs_world = ECSWorld::new();

        let mut render_scene = RenderScene::new(StaticRenderState {
            device: self.device.clone(),
            queue: self.queue.clone(),
            bind_group_layouts: Vec::new(),
        });

        CameraEntity::add_flying(
            &mut ecs_world, &mut render_scene,
            Vec3A::new(0.0, 0.0, -5.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 0.0, 1.0),
            Vec3A::new(0.0, 1.0, 0.0),
            70.0,
            0.01,
            None,
            1.0,
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
        self.engine_core_state = Some(EngineCoreState { render_pipeline, triangle_render_bundle, render_scene, ecs_world: ecs_world, input_handler: InputHandler::new() });
    }

    /// Performs the pre-render phase of the engine.
    /// This includes updating the ECS world.
    /// [render_camera_ecs_handle] is the handle of the camera ECS entity to use for the render.
    #[profiling::function]
    fn pre_render(&mut self, delta_time: f64, render_camera_ecs_handle: ECSEntityHandle) {
        let engine_state: &mut EngineCoreState = self.engine_core_state.as_mut().unwrap();

        let render_camera_node_handle = engine_state.get_render_node_handle_by_ecs_handle(&render_camera_ecs_handle).unwrap().clone();
        let render_scene = &mut engine_state.render_scene;

        // Mark render_camera as the active camera.
        {
            render_scene.set_active_camera(&render_camera_node_handle);
        }

        // Input to movement_input
        {
            let input_handler = &mut engine_state.input_handler;
            let primrary_keyboard_opt = input_handler.get_primary_keyboard();
            if let Some(keyboard) = primrary_keyboard_opt {
                self.movement_input.forward = keyboard.is_key_pressed(VirtualKeyCode::W);
                self.movement_input.backward = keyboard.is_key_pressed(VirtualKeyCode::S);
                self.movement_input.left = keyboard.is_key_pressed(VirtualKeyCode::A);
                self.movement_input.right = keyboard.is_key_pressed(VirtualKeyCode::D);
                self.movement_input.up = keyboard.is_key_pressed(VirtualKeyCode::Space);
                self.movement_input.down = keyboard.is_key_pressed(VirtualKeyCode::LShift);
            }
        }

        // Pre-render phase
        // TODO: MOVE OFF RENDER THREAD
        {
            engine_state.ecs_world.update(delta_time, self.movement_input.clone(), render_scene);
        }

        self.movement_input.new_frame();
    }

    #[profiling::function]
    pub fn render<'a, 'b: 'a>(&'b mut self, command_encoder: &'a mut CommandEncoder, surface_texture_view: &wgpu::TextureView, mutisampled_framebuffer: Option<&wgpu::TextureView>, viewport_region: &ViewportRegion, render_camera_handle: ECSEntityHandle, delta_time: f64) {
        if viewport_region == &ViewportRegion::ZERO || self.engine_core_state.is_none() {
            return;
        }

        // Pre-render phase (TODO: MOVE OFF RENDER THREAD)
        self.pre_render(delta_time, render_camera_handle);

        let engine_state: &mut EngineCoreState = self.engine_core_state.as_mut().unwrap();

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
        let ecs_world = &engine_state.ecs_world;
        for camera_ecs_handle in ecs_world.get_cameras() {
            let camera_rende_node_handle = ecs_world.get_entity(camera_ecs_handle).unwrap()
                .get_render_node()
                .unwrap();
            let camera_node: &mut CameraRenderNode = engine_state.render_scene.get_node_by_id(camera_rende_node_handle).unwrap();
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
    pub fn handle_mouse_button_event(&mut self, _device_id: DeviceId, mouse_button: MouseButton, button_state: ElementState, _delta_time: f64) {
        if button_state == ElementState::Pressed && mouse_button == MouseButton::Middle {
            self.movement_input.should_roll = true;
        } else if button_state == ElementState::Released && mouse_button == MouseButton::Middle {
            self.movement_input.should_roll = false;
        }
    }

    #[profiling::function]
    pub fn handle_mouse_wheel(&mut self, _device_id: DeviceId, _delta: MouseScrollDelta, _phase: TouchPhase, _delta_time: f64) {}

    #[profiling::function]
    pub fn handle_mouse_motion(&mut self, _device_id: DeviceId, mouse_delta: (f64, f64), _delta_time: f64) {
        // TODO: CONFIGURABLE MOUSE SENSITIVITY
        self.movement_input.delta_yaw += mouse_delta.0 as f32 / 1000.0;
        self.movement_input.delta_pitch += mouse_delta.1 as f32 / 1000.0;
    }

    pub fn should_grab_cursor(&self) -> bool {
        return true;
    }
}