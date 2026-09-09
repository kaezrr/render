#![feature(file_buffered)]

mod camera;
mod hdr;
mod instance;
mod light;
mod model;
mod parser;
mod pipeline;
mod state;
mod texture;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use log::error;
use log::info;
use log::warn;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event::MouseButton;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::Window;
use winit::window::WindowId;

use crate::state::State;

#[derive(Debug)]
pub struct App {
    state: Option<State<'static>>,
    last_frame_time: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: None,
            last_frame_time: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let attributes = Window::default_attributes()
                .with_title("Render - PROJECT")
                .with_inner_size(LogicalSize::new(800, 600));

            Arc::new(
                event_loop
                    .create_window(attributes)
                    .expect("window should be initialized"),
            )
        };

        self.state = Some(
            pollster::block_on(State::new(window)).expect("renderer state should be initialized"),
        );

        info!("Window initialized!");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            warn!("Ignoring window event because app is not ready yet!");
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Closing application...");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now - self.last_frame_time;
                self.last_frame_time = now;
                state.update(dt);

                if let Err(e) = state.render() {
                    error!("Error while rendering: {e}");
                    event_loop.exit();
                }
            }

            WindowEvent::Resized(size) => {
                state.resize_surface(size.width, size.height);
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if key == KeyCode::Escape && key_state == ElementState::Pressed {
                    event_loop.exit();
                } else {
                    state.process_keyboard(key, key_state.is_pressed());
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                state.process_mouse_scroll(&delta);
            }

            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } if button_state.is_pressed() => match button {
                MouseButton::Left => {
                    if let Err(err) = state.capture_mouse() {
                        error!("Could not grab cursor: {err}");
                    }
                }
                MouseButton::Right => {
                    if let Err(err) = state.release_mouse() {
                        error!("Could not release cursor: {err}");
                    }
                }
                _ => {}
            },

            WindowEvent::Focused(false) => {
                if let Err(err) = state.release_mouse() {
                    error!("Could not release cursor: {err}");
                }
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        let Some(state) = self.state.as_mut() else {
            warn!("Ignoring device event because app is not ready yet!");
            return;
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            state.process_mouse_delta(delta.0, delta.1);
        }
    }
}

pub(crate) fn load_asset_bytes(file_name: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let asset_path = create_asset_path(file_name);
    std::fs::read(asset_path)
}

pub(crate) fn load_asset_string(file_name: impl AsRef<Path>) -> std::io::Result<String> {
    let asset_path = create_asset_path(file_name);
    std::fs::read_to_string(asset_path)
}

pub(crate) fn create_asset_path(file_name: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file_name)
}
