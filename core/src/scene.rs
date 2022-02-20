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
    // The camera's up vector.
    camera_up: Vec3,
    // The camera's aspect ratio.
    aspect: f32,
    // The camera's vertical field of view.
    fov: f32,
    // The camera's near plane.
    near: f32,
    // The camera's far plane.
    far: f32,

    // The state of the camera passed to the shader for vertex space transformation
    pub camera_shader_state: CameraShaderState,
}

impl Camera {

    pub fn new(position: Vec3, direction: Vec3, fov: f32, near: f32, far: f32, aspect: f32, up_axis: Vec3) -> Camera {
        let right = up_axis.cross(direction);
        let camera_up = direction.cross(right);
        return Camera {
            position: position,
            direction: direction,
            up_axis: up_axis,
            camera_up: camera_up,
            aspect: aspect,
            fov: fov,
            near: near,
            far: far,
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
        let camera_right = self.up_axis.cross(self.direction);
        self.camera_up = self.direction.cross(camera_right);

        let view_matrix = Mat4::look_at_lh(self.position, self.position + self.direction, self.camera_up);
        let projection_matrix = Mat4::perspective_lh(self.fov, self.aspect, self.near, self.far);
        self.camera_shader_state.view_proj = (projection_matrix * view_matrix).to_cols_array_2d();
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_direction(&mut self, direction: Vec3) {
        self.direction = direction;
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
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
    pub fn camera_up(&self) -> Vec3 {
        self.camera_up
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