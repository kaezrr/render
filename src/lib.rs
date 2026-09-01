mod camera;
mod consts;
mod primitives;
mod state;
mod texture;

use std::sync::Arc;

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

#[derive(Default)]
pub struct App {
    state: Option<State<'static>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let attributes = Window::default_attributes()
                .with_title("Render - PROJECT")
                .with_inner_size(LogicalSize::new(800, 600));

            Arc::new(event_loop.create_window(attributes).unwrap())
        };

        let state = pollster::block_on(State::new(window)).unwrap();
        self.state = Some(state);

        info!("Window initialized!")
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            warn!("Window not initialized yet!");
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Closing application...");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                state.update();

                if let Err(e) = state.render() {
                    error!("Error while rendering: {e}");
                    event_loop.exit();
                }
            }

            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
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

macro_rules! asset_str {
    ($path:expr) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path))
    };
}

macro_rules! asset_bytes {
    ($path:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path))
    };
}

pub(crate) use asset_bytes;
pub(crate) use asset_str;
