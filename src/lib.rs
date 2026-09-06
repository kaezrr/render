mod camera;
mod instance;
mod model;
mod parser;
mod pipeline;
mod state;
mod texture;

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use log::error;
use log::info;
use log::warn;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::KeyEvent;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
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
    #[expect(
        clippy::expect_used,
        reason = "Dont really have a good way of handling error here so rather crash"
    )]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let attributes = Window::default_attributes()
                .with_title("Render - PROJECT")
                .with_inner_size(LogicalSize::new(800, 600));

            Arc::new(
                event_loop
                    .create_window(attributes)
                    .expect("window successfully initialized"),
            )
        };

        self.state = Some(
            pollster::block_on(State::new(window))
                .expect("renderer state successfully initialized"),
        );

        info!("Window initialized!");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            warn!("Window not initialized yet!");
            return;
        };

        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        match event {
            WindowEvent::CloseRequested => {
                info!("Closing application...");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
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
                        physical_key: winit::keyboard::PhysicalKey::Code(key),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                state.handle_key(event_loop, key, key_state.is_pressed());
            }

            _ => (),
        }
    }
}

pub(crate) fn load_asset_bytes(file_name: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let asset_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file_name);

    std::fs::read(asset_path)
}

pub(crate) fn load_asset_string(file_name: impl AsRef<Path>) -> std::io::Result<String> {
    let asset_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file_name);

    std::fs::read_to_string(asset_path)
}
