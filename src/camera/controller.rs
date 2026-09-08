use core::f32;
use core::time::Duration;

use glam::Vec3;
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

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
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 300.0,
            MouseScrollDelta::PixelDelta(physical_position) => physical_position.y as f32,
        }
    }

    pub fn update(&mut self, camera: &mut super::Camera, dt: Duration) {
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
