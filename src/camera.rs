use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat4;
use glam::camera;

pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect_ratio: f32,
    pub vertical_fov: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = camera::rh::view::look_at_mat4(self.eye, self.target, self.up);
        let projection = camera::rh::proj::directx::perspective(
            self.vertical_fov,
            self.aspect_ratio,
            self.znear,
            self.zfar,
        );

        projection * view
    }

    pub fn create_uniform(&self) -> CameraUniform {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_projection(self);
        camera_uniform
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    view_projection: [f32; 16],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection: Mat4::IDENTITY.to_cols_array(),
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().to_cols_array();
    }
}
