use core::f32;
use core::time::Duration;

use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat4;
use glam::Vec3;
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
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

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

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,

    amount_up: f32,
    amount_down: f32,

    rotate_horizontal: f32,
    rotate_vertical: f32,

    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,

            amount_up: 0.0,
            amount_down: 0.0,

            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,

            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, is_pressed: bool) {
        let amount = if is_pressed { 1.0 } else { 0.0 };

        match key {
            KeyCode::KeyW => self.amount_forward = amount,
            KeyCode::KeyA => self.amount_left = amount,
            KeyCode::KeyS => self.amount_backward = amount,
            KeyCode::KeyD => self.amount_right = amount,
            KeyCode::ControlLeft => self.amount_up = amount,
            KeyCode::ShiftLeft => self.amount_down = amount,
            x => log::debug!("Ignoring keypress: {x:?}"),
        }
    }

    pub fn process_mouse_delta(&mut self, dx: f64, dy: f64) {
        self.rotate_horizontal = dx as f32;
        self.rotate_vertical = dy as f32;
    }

    pub fn process_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(physical_position) => physical_position.y as f32,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        const SAFETY_BOUND: f32 = f32::consts::FRAC_PI_2 - 0.0001;

        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        // Move forward, backward, left or right
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward = Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

        // Pseudozooming via mouse scroll
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        // Reset this to zero so the camera only zooms on active mouse scroll
        self.scroll = 0.0;

        // Move up or down
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += self.rotate_horizontal.to_radians() * self.sensitivity;
        camera.pitch += -self.rotate_vertical.to_radians() * self.sensitivity;
        // Reset these to zero so the camera only rotates on active mouse movement
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Bound the camera vertical rotation
        camera.pitch = camera.pitch.clamp(-SAFETY_BOUND, SAFETY_BOUND);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    view_projection: [f32; 16],
}

impl CameraUniform {
    pub const fn new() -> Self {
        Self {
            view_projection: Mat4::IDENTITY.to_cols_array(),
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera, projection: &Projection) {
        let view = camera.matrix();
        let proj = projection.matrix();
        self.view_projection = (proj * view).to_cols_array();
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
        let mut uniform = CameraUniform::new();
        uniform.update_view_projection(&camera, &projection);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera_binding_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
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
        self.controller.update_camera(&mut self.camera, dt);
        self.uniform
            .update_view_projection(&self.camera, &self.projection);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}
