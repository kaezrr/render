use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat4;
use glam::camera;
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
use winit::keyboard::KeyCode;

#[derive(Debug)]
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
    pub const fn new() -> Self {
        Self {
            view_projection: Mat4::IDENTITY.to_cols_array(),
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().to_cols_array();
    }
}

#[derive(Debug)]
pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    const fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    const fn handle_key(&mut self, code: KeyCode, is_pressed: bool) {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.is_forward_pressed = is_pressed,
            KeyCode::KeyS | KeyCode::ArrowDown => self.is_backward_pressed = is_pressed,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.is_left_pressed = is_pressed,
            KeyCode::KeyD | KeyCode::ArrowRight => self.is_right_pressed = is_pressed,
            _ => (),
        }
    }

    fn update_camera(&self, camera: &mut Camera) {
        let (forward, forward_magnitude) = (camera.target - camera.eye).normalize_and_length();

        if self.is_forward_pressed && forward_magnitude > self.speed {
            camera.eye += forward * self.speed;
        }

        if self.is_backward_pressed {
            camera.eye -= forward * self.speed;
        }

        let right = forward.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_magnitude = forward.length();

        if self.is_right_pressed {
            camera.eye =
                camera.target - (forward + right * self.speed).normalize() * forward_magnitude;
        }

        if self.is_left_pressed {
            camera.eye =
                camera.target - (forward - right * self.speed).normalize() * forward_magnitude;
        }
    }
}

#[derive(Debug)]
pub struct CameraBundle {
    pub camera: Camera,
    pub controller: CameraController,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,

    buffer: Buffer,
    uniform: CameraUniform,
}

impl CameraBundle {
    pub fn new(device: &Device, camera: Camera, speed: f32) -> Self {
        let uniform = camera.create_uniform();

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
            controller: CameraController::new(speed),
            bind_group,
            bind_group_layout,
            buffer,
            uniform,
        }
    }

    pub fn update(&mut self, queue: &Queue) {
        self.controller.update_camera(&mut self.camera);
        self.uniform.update_view_projection(&self.camera);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub const fn handle_key(&mut self, code: KeyCode, is_pressed: bool) {
        self.controller.handle_key(code, is_pressed);
    }
}
