use std::sync::Arc;

use log::info;
use log::warn;
use wgpu::Backends;
use wgpu::Color;
use wgpu::Device;
use wgpu::ExperimentalFeatures;
use wgpu::Features;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::Operations;
use wgpu::PresentMode::AutoNoVsync;
use wgpu::Queue;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RequestAdapterOptionsBase;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::TextureUsages;
use wgpu::wgt::CommandEncoderDescriptor;
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
    surface_configuration: SurfaceConfiguration,
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

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("wgpu state device"),
                required_features: Features::empty(),
                required_limits: Default::default(),
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        info!("Using physical device: {}", device.adapter_info().name);

        let surface_configuration = {
            let capabilites = surface.get_capabilities(&adapter);

            let surface_format = capabilites
                .formats
                .iter()
                .find(|x| x.is_srgb())
                .copied()
                .unwrap_or(capabilites.formats[0]);

            let size = window.inner_size();

            SurfaceConfiguration {
                usage: capabilites.usages,
                format: surface_format,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: size.width,
                height: size.height,
                present_mode: capabilites.present_modes[0],
                desired_maximum_frame_latency: 2,
                alpha_mode: capabilites.alpha_modes[0],
                view_formats: vec![],
            }
        };

        surface.configure(&device, &surface_configuration);

        Ok(Self {
            instance,
            device,
            window,
            queue,
            surface,
            surface_configuration,
        })
    }

    async fn render(&self) -> anyhow::Result<()> {
        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let current_texture = self.surface.get_current_texture();

        let render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: todo!(),
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let command_buffer = command_encoder.finish();

        self.queue.submit([command_buffer]);

        Ok(())
    }

    fn resize(&mut self, new_width: u32, new_height: u32) {
        self.surface_configuration.width = new_width;
        self.surface_configuration.height = new_height;

        self.surface
            .configure(&self.device, &self.surface_configuration);
    }
}
