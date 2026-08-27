use std::sync::Arc;

use log::debug;
use log::info;
use log::warn;
use wgpu::Backends;
use wgpu::BlendState;
use wgpu::Color;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::Device;
use wgpu::ExperimentalFeatures;
use wgpu::Face;
use wgpu::Features;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::Queue;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::RequestAdapterOptionsBase;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::TextureUsages;
use wgpu::VertexState;
use wgpu::wgt::CommandEncoderDescriptor;
use wgpu::wgt::DeviceDescriptor;
use wgpu::wgt::TextureViewDescriptor;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

pub struct State {
    pub window: Arc<Window>,

    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: RenderPipeline,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
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
                usage: TextureUsages::RENDER_ATTACHMENT,
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

        let render_pipeline = {
            let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

            let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("render pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },

                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },

                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },

                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: surface_configuration.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),

                multiview_mask: None,
                cache: None,
            })
        };

        Ok(Self {
            device,
            window,
            queue,
            surface,
            config: surface_configuration,
            is_surface_configured: false,
            render_pipeline,
        })
    }

    pub fn render(&self) -> anyhow::Result<()> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            warn!("Trying to render unconfigured surface");
            return Ok(());
        }

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let current_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,

            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,

            wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("Device was lost"),

            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }

            _ => return Ok(()),
        };

        let texture_view = current_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(command_encoder.finish()));
        self.queue.present(current_texture);

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, is_pressed: bool) {
        match (key, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (key, is_pressed) => debug!("{key:?} detected, pressed: {is_pressed}"),
        }
    }
}
