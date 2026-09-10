mod controller;

use core::f32;
use core::time::Duration;

use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat4;
use glam::Vec3;
use glam::Vec4;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::Buffer;
use wgpu::BufferUsages;
use wgpu::Device;
use wgpu::Queue;
use wgpu::ShaderStages;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::camera::controller::CameraController;

#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    /// Create a new camera with a given position, yaw and pitch.
    /// yaw and pitch are in degrees
    pub fn new(position: impl Into<Vec3>, yaw: f32, pitch: f32) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.to_radians(),
            pitch: pitch.to_radians(),
        }
    }

    pub fn matrix(&self) -> Mat4 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();

        glam::camera::rh::view::look_to_mat4(
            self.position,
            Vec3 {
                x: cos_pitch * cos_yaw,
                y: sin_pitch,
                z: cos_pitch * sin_yaw,
            }
            .normalize(),
            Vec3::Y,
        )
    }
}

#[derive(Debug)]
pub struct Projection {
    aspect_ratio: f32,
    vertical_fov: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect_ratio: width as f32 / height as f32,
            vertical_fov: fovy.to_radians(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    pub fn matrix(&self) -> Mat4 {
        glam::camera::rh::proj::directx::perspective(
            self.vertical_fov,
            self.aspect_ratio,
            self.znear,
            self.zfar,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    view_position: Vec4,
    view: Mat4,
    inv_view: Mat4,
    view_projection: Mat4,
    inv_projection: Mat4,
}

impl CameraUniform {
    pub fn new(camera: &Camera, projection: &Projection) -> Self {
        let mut uniform = Self {
            view_position: Vec4::ZERO,
            view: Mat4::IDENTITY,
            inv_view: Mat4::IDENTITY,
            view_projection: Mat4::IDENTITY,
            inv_projection: Mat4::IDENTITY,
        };

        uniform.update(camera, projection);
        uniform
    }

    pub fn update(&mut self, camera: &Camera, projection: &Projection) {
        let view = camera.matrix();
        let projection = projection.matrix();

        self.view_position = camera.position.to_homogeneous();

        self.view = view;
        self.inv_view = view.inverse();

        self.view_projection = projection * view;
        self.inv_projection = projection.inverse();
    }
}

#[derive(Debug)]
pub struct CameraBundle {
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    pub controller: CameraController,
    pub projection: Projection,

    camera: Camera,
    buffer: Buffer,
    uniform: CameraUniform,
}

impl CameraBundle {
    pub fn new(
        device: &Device,
        camera: Camera,
        projection: Projection,
        speed: f32,
        sensitivity: f32,
    ) -> Self {
        let uniform = CameraUniform::new(&camera, &projection);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera_binding_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("camera_binding_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            camera,
            controller: CameraController::new(speed, sensitivity),
            bind_group,
            bind_group_layout,
            projection,
            buffer,
            uniform,
        }
    }

    pub fn update(&mut self, queue: &Queue, dt: Duration) {
        self.controller.update(&mut self.camera, dt);
        self.uniform.update(&self.camera, &self.projection);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}
