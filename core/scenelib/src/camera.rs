use std::any::Any;
use std::f32::consts::PI;
use glam::{Mat4, Quat, Vec3, Vec3A};
use wgpu::util::DeviceExt;
use crate::scene::{StaticRenderState, RenderNode, RenderScene, RenderCallState, RenderNodeHandle};

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraShaderState {
    // The cameras view matrix and projection matrix multiplied together
    view_proj: [[f32; 4]; 4],
}

pub struct PerspectiveCamera {
    // The camera's position.
    position: Vec3A,
    // The camera's direction.
    direction: Vec3A,
    // The camera's right vector.
    right: Vec3A,
    // The camera's forward axis.
    forward_axis: Vec3A,
    // The camera's up axis
    up_axis: Vec3A,
    // The camera's up vector
    up: Vec3A,
    // The camera's aspect ratio.
    aspect: f32,
    // The camera's vertical field of view.
    fov: f32,
    // The camera's near plane.
    near: f32,
    // The camera's far plane.
    far: Option<f32>,
    // Whether the camera's state has changed since last frame
    dirty: bool,
    // The state of the camera passed to the shader for vertex space transformation
    pub camera_shader_state: CameraShaderState,
}

impl PerspectiveCamera {
    pub fn new(position: Vec3A, direction: Vec3A, forward_axis: Vec3A, up_axis: Vec3A, fov_degrees: f32, near: f32, far: Option<f32>, aspect: f32) -> PerspectiveCamera {
        return PerspectiveCamera {
            position: position,
            direction: direction,
            right: up_axis.cross(direction),
            forward_axis: forward_axis,
            up_axis: up_axis,
            up: direction.cross(up_axis.cross(direction)),
            aspect: aspect,
            fov: fov_degrees.to_radians(),
            near: near,
            far: far,
            dirty: true,
            camera_shader_state: CameraShaderState {
                view_proj: [
                    // Hardcoded identity lol because bytemuck
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ]
            },
        };
    }

    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }

        let view_matrix = Mat4::look_at_lh(Vec3::from(self.position), Vec3::from(self.position + self.direction), Vec3::from(self.up));

        let projection_matrix = match self.far {
            Some(far) => Mat4::perspective_lh(self.fov, self.aspect, self.near, far),
            None => Mat4::perspective_infinite_lh(self.fov, self.aspect, self.near),
        };

        self.camera_shader_state.view_proj = (projection_matrix * view_matrix).to_cols_array_2d();
        self.dirty = false;
    }

    pub fn set_up_axis(&mut self, up_axis: Vec3A) {
        if self.up_axis == up_axis {
            return;
        }
        self.up_axis = up_axis;
        self.up = self.direction.cross(self.up_axis.cross(self.direction));
        self.dirty = true;
    }

    pub fn set_position(&mut self, position: Vec3A) {
        if self.position == position {
            return;
        }
        self.position = position;
        self.dirty = true;
    }

    pub fn set_direction(&mut self, direction: Vec3A) {
        if self.direction == direction {
            return;
        }
        self.direction = direction;
        self.right = self.up_axis.cross(self.direction);
        self.up = self.direction.cross(self.right.cross(self.direction));
        self.dirty = true;
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        if self.aspect == aspect {
            return;
        }
        self.aspect = aspect;
        self.dirty = true;
    }

    pub fn set_fov(&mut self, fov: f32) {
        if self.fov == fov {
            return;
        }
        self.fov = fov;
        self.dirty = true;
    }

    pub fn set_near(&mut self, near: f32) {
        if self.near == near {
            return;
        }
        self.near = near;
        self.dirty = true;
    }

    pub fn set_far(&mut self, far: f32) {
        if self.far == Some(far) {
            return;
        }
        self.far = Some(far);
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        let direction = rotation * self.forward_axis;
        let up = rotation * self.up_axis;
        let right = up.cross(direction);

        if self.direction != direction || self.up != up || self.right != right {
            self.dirty = true;
        }
        self.direction = direction;
        self.up = up;
        self.right = right;
    }

    pub fn set_rotation_euler(&mut self, yaw_degrees: f32, pitch_degrees: f32) {
        self.direction.x = yaw_degrees.to_radians().cos() * pitch_degrees.to_radians().cos();
        self.direction.y = pitch_degrees.to_radians().sin();
        self.direction.z = yaw_degrees.to_radians().sin() * pitch_degrees.to_radians().cos();
        self.right = self.up_axis.cross(self.direction);
        self.up = self.direction.cross(self.right);
        self.dirty = true;
    }

    pub fn set_roll_euler(&mut self, roll_degrees: f32) {
        self.up_axis.x = roll_degrees.to_radians().cos();
        self.up_axis.y = roll_degrees.to_radians().sin();
        self.right = self.up_axis.cross(self.direction);
        self.up = self.direction.cross(self.right);
        self.dirty = true;
    }

    pub fn yaw(&self) -> f32 {
        return self.direction.z.atan2(self.direction.x) * (180.0_f32 / PI);
    }
    pub fn pitch(&self) -> f32 {
        return self.direction.y.asin() * (180.0_f32 / PI);
    }
    pub fn roll(&self) -> f32 {
        return self.up_axis.y.atan2(self.up_axis.x) * (180.0_f32 / PI);
    }
    pub fn position(&self) -> Vec3A {
        self.position
    }
    pub fn direction(&self) -> Vec3A {
        self.direction
    }
    pub fn up_axis(&self) -> Vec3A {
        self.up_axis
    }
    pub fn aspect(&self) -> f32 {
        self.aspect
    }
    pub fn fov(&self) -> f32 {
        self.fov
    }
    pub fn near(&self) -> f32 {
        self.near
    }
    pub fn far(&self) -> Option<f32> {
        self.far
    }
    pub fn right(&self) -> Vec3A {
        self.right
    }
    pub fn up(&self) -> Vec3A {
        self.up
    }
}

pub struct CameraRenderNode {
    camera: PerspectiveCamera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    /// State whether the camera is currently the camera being rendered from.
    /// Only one camera can be rendered from at a time. This is used for split screen.
    is_active_camera: bool,

    dirty: bool,
}

impl CameraRenderNode {
    pub fn set_active(&mut self) {
        if self.is_active_camera {
            return;
        }
        self.is_active_camera = true;
        self.dirty = true;
    }

    pub fn set_inactive(&mut self) {
        if !self.is_active_camera {
            return;
        }
        self.is_active_camera = false;
        self.dirty = true;
    }
}

impl RenderNode for CameraRenderNode {
    fn is_dirty(&self) -> bool {
        return self.dirty || self.camera.dirty;
    }

    #[profiling::function]
    fn render<'a, 'b: 'a>(&'b mut self, _static_render_state: &mut StaticRenderState, render_call: &mut RenderCallState<'_, 'b>) {
        render_call.render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
    }

    fn resolve_dirty_state(&mut self, static_render_state: &mut StaticRenderState) {
        let was_camera_dirty = self.camera.dirty;
        let was_camera_render_node_dirty = self.dirty;
        if self.camera.dirty {
            self.camera.update();
        }
        if self.dirty {
            self.dirty = false;
        }
        if self.is_active_camera {
            if was_camera_dirty || was_camera_render_node_dirty {
                // If the camera's state has changed OR this camera has just become the active camera, meaning that
                // the contents of self.camera_buffer are some other camera's data, we need to update the camera buffer
                static_render_state.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera.camera_shader_state]));
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl CameraRenderNode {
    pub fn add_new(camera: PerspectiveCamera, scene: &mut RenderScene) -> RenderNodeHandle {
        let render_context = &mut scene.static_render_state;
        let camera_buffer = render_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("CameraBuffer"),
                contents: bytemuck::cast_slice(&[camera.camera_shader_state]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = render_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = render_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        render_context.push_bind_group_layout(camera_bind_group_layout);
        let camera_node = CameraRenderNode {
            camera,
            camera_buffer,
            camera_bind_group,
            is_active_camera: false,
            dirty: false,
        };
        return scene.add_node(Box::new(camera_node));
    }

    pub fn set_position(&mut self, position: Vec3A) {
        self.camera.set_position(position);
    }

    pub fn set_rotation(&mut self, quat: Quat) {
        self.camera.set_rotation(quat);
    }

    pub fn set_rotation_euler(&mut self, yaw_degrees: f32, pitch_degrees: f32) {
        self.camera.set_rotation_euler(yaw_degrees, pitch_degrees);
    }

    pub fn set_roll_euler(&mut self, roll_degrees: f32) {
        self.camera.set_roll_euler(roll_degrees);
    }

    pub fn yaw(&self) -> f32 {
        return self.camera.yaw();
    }

    pub fn pitch(&self) -> f32 {
        return self.camera.pitch();
    }

    pub fn roll(&self) -> f32 {
        return self.camera.roll();
    }

    pub fn position(&self) -> Vec3A {
        return self.camera.position();
    }

    pub fn direction(&self) -> Vec3A {
        return self.camera.direction();
    }

    pub fn right(&self) -> Vec3A {
        return self.camera.right();
    }

    pub fn up(&self) -> Vec3A {
        return self.camera.up();
    }

    pub fn up_axis(&self) -> Vec3A {
        return self.camera.up_axis();
    }

    pub fn aspect(&self) -> f32 {
        return self.camera.aspect();
    }

    pub fn fov(&self) -> f32 {
        return self.camera.fov();
    }

    pub fn near(&self) -> f32 {
        return self.camera.near();
    }

    pub fn far(&self) -> Option<f32> {
        return self.camera.far();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.camera.set_aspect(aspect);
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.camera.set_fov(fov);
    }

    pub fn set_up_axis(&mut self, up_axis: Vec3A) {
        self.camera.set_up_axis(up_axis);
    }

    pub fn set_near(&mut self, near: f32) {
        self.camera.set_near(near);
    }

    pub fn set_far(&mut self, far: f32) {
        self.camera.set_far(far);
    }

    pub fn update(&mut self) {
        self.camera.update();
    }
}
