use std::f32::consts::PI;
use glam::{Mat4, Vec3};

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraShaderState {
    // The cameras view matrix and projection matrix multiplied together
    view_proj: [[f32; 4]; 4],
}

pub struct Camera {
    // The camera's position.
    position: Vec3,
    // The camera's direction.
    direction: Vec3,
    // The camera's up axis
    up_axis: Vec3,
    // The camera's aspect ratio.
    aspect: f32,
    // The camera's vertical field of view.
    fov: f32,
    // The camera's near plane.
    near: f32,
    // The camera's far plane.
    far: f32,
    // Whether the camera's state has changed since last frame
    dirty: bool,
    // The state of the camera passed to the shader for vertex space transformation
    pub camera_shader_state: CameraShaderState,
}

impl Camera {
    pub fn new(position: Vec3, direction: Vec3, fov: f32, near: f32, far: f32, aspect: f32, up_axis: Vec3) -> Camera {
        return Camera {
            position: position,
            direction: direction,
            up_axis: up_axis,
            aspect: aspect,
            fov: fov,
            near: near,
            far: far,
            dirty: true,
            camera_shader_state: CameraShaderState {
                view_proj: [
                    // Hardcoded identity lol because bytemuck
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]
                ]
            },
        };
    }

    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }
        let camera_right = self.up_axis.cross(self.direction);
        let camera_up = self.direction.cross(camera_right);

        let view_matrix = Mat4::look_at_lh(self.position, self.position + self.direction, camera_up);
        let projection_matrix = Mat4::perspective_lh(self.fov, self.aspect, self.near, self.far);
        self.camera_shader_state.view_proj = (projection_matrix * view_matrix).to_cols_array_2d();
    }

    pub fn set_position(&mut self, position: Vec3) {
        if self.position == position {
            return;
        }
        self.position = position;
        self.dirty = true;
    }

    pub fn set_direction(&mut self, direction: Vec3) {
        if self.direction == direction {
            return;
        }
        self.direction = direction;
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
        if self.far == far {
            return;
        }
        self.far = far;
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, yaw_degrees: f32, pitch_degrees: f32) {
        self.direction.x = yaw_degrees.to_radians().cos() * pitch_degrees.to_radians().cos();
        self.direction.y = pitch_degrees.to_radians().sin();
        self.direction.z = yaw_degrees.to_radians().sin() * pitch_degrees.to_radians().cos();
        self.dirty = true;
    }

    pub fn set_roll(&mut self, roll_degrees: f32) {
        self.up_axis.x = roll_degrees.to_radians().cos();
        self.up_axis.y = roll_degrees.to_radians().sin();
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

    pub fn position(&self) -> Vec3 {
        self.position
    }
    pub fn direction(&self) -> Vec3 {
        self.direction
    }
    pub fn up_axis(&self) -> Vec3 {
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
    pub fn far(&self) -> f32 {
        self.far
    }
}