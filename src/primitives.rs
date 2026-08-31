#![allow(unused)]

use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat3A;
use glam::Mat4;
use glam::camera;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl<'a> Vertex {
    const ATTRIBS: [VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];

    pub fn desc() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub texture_coordinates: [f32; 2],
}

impl<'a> TexturedVertex {
    const ATTRIBS: [VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
    ];

    pub fn desc() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

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

    pub fn with_view_projection(mut self, camera: &Camera) -> Self {
        self.update_view_projection(camera);
        self
    }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().to_cols_array();
    }
}
