use std::sync::Arc;

use log::info;
use log::warn;
use wgpu::Backends;
use wgpu::Device;
use wgpu::ExperimentalFeatures;
use wgpu::Features;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::Limits;
use wgpu::MemoryHints;
use wgpu::Queue;
use wgpu::RequestAdapterOptionsBase;
use wgpu::Surface;
use wgpu::wgt::DeviceDescriptor;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use winit::window::WindowId;

#[derive(Default)]
pub struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let attributes = Window::default_attributes()
                .with_title("Render")
                .with_inner_size(LogicalSize::new(640, 480));

            Arc::new(event_loop.create_window(attributes).unwrap())
        };

        let state = pollster::block_on(State::new(window)).unwrap();
        self.state = Some(state);

        info!("Window initialized!")
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_ref() else {
            warn!("Window not initialized yet!");
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Closing application...");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                state.window.request_redraw();
            }

            _ => (),
        }
    }
}

struct State {
    instance: Instance,
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone())?;

        let (device, queue) = {
            let adapter = instance
                .request_adapter(&RequestAdapterOptionsBase {
                    power_preference: wgpu::PowerPreference::None,
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                    apply_limit_buckets: false,
                })
                .await?;

            info!("Using physical device: {}", adapter.get_info().name);

            adapter
                .request_device(&DeviceDescriptor {
                    label: Some("wgpu state device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    experimental_features: ExperimentalFeatures::disabled(),
                    memory_hints: MemoryHints::default(),
                    trace: wgpu::Trace::Off,
                })
                .await?
        };

        Ok(Self {
            instance,
            device,
            window,
            queue,
            surface,
        })
    }
}
