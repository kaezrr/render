mod gpu;

use std::sync::Arc;

use log::warn;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BlendState;
use wgpu::Buffer;
use wgpu::BufferUsages;
use wgpu::Color;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::Device;
use wgpu::Face;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::ShaderModuleDescriptor;
use wgpu::ShaderStages;
use wgpu::VertexState;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::wgt::CommandEncoderDescriptor;
use wgpu::wgt::TextureViewDescriptor;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::asset_bytes;
use crate::asset_str;
use crate::camera::Camera;
use crate::camera::CameraBundle;
use crate::consts::INDICES;
use crate::consts::TEXTURED_VERTICES;
use crate::primitives::TexturedVertex;
use crate::state::gpu::GpuContext;
use crate::texture::Texture;

pub struct State<'a> {
    pub window: Arc<Window>,
    pub gpu_context: GpuContext<'a>,

    render_pipeline: RenderPipeline,

    mesh: Mesh,

    diffuse_bind_group: BindGroup,
    diffuse_texture: Texture,

    camera: CameraBundle,
}

impl State<'_> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu_context = GpuContext::new(window.clone()).await?;

        let diffuse_bytes = asset_bytes!("happy-tree.png");
        let diffuse_texture = Texture::from_bytes(
            &gpu_context.device,
            &gpu_context.queue,
            diffuse_bytes,
            "happy_tree_texture",
        )?;

        let texture_bind_group_layout =
            gpu_context
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("texture_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let diffuse_bind_group = gpu_context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let camera = CameraBundle::new(
            &gpu_context.device,
            Camera {
                eye: (0.0, 1.0, 2.0).into(),
                target: (0.0, 0.0, 0.0).into(),
                up: glam::Vec3::Y,
                aspect_ratio: gpu_context.config.width as f32 / gpu_context.config.height as f32,
                vertical_fov: 45.0f32.to_radians(),
                znear: 0.1,
                zfar: 100.0,
            },
            0.005,
        );

        let render_pipeline = {
            let shader_module = gpu_context
                .device
                .create_shader_module(ShaderModuleDescriptor {
                    label: Some("texture.wgsl"),
                    source: wgpu::ShaderSource::Wgsl(asset_str!("shaders/texture.wgsl").into()),
                });

            let render_pipeline_layout =
                gpu_context
                    .device
                    .create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("render pipeline layout"),
                        bind_group_layouts: &[
                            Some(&texture_bind_group_layout),
                            Some(&camera.bind_group_layout),
                        ],
                        immediate_size: 0,
                    });

            gpu_context
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("render pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: VertexState {
                        module: &shader_module,
                        entry_point: Some("vs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[Some(TexturedVertex::desc())],
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
                            format: gpu_context.config.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })],
                    }),

                    multiview_mask: None,
                    cache: None,
                })
        };

        let mesh = Mesh::new(&gpu_context.device, TEXTURED_VERTICES, INDICES);

        Ok(Self {
            window,
            gpu_context,

            render_pipeline,

            mesh,

            diffuse_bind_group,
            diffuse_texture,

            camera,
        })
    }

    pub fn render(&self) -> anyhow::Result<()> {
        self.window.request_redraw();

        if !self.gpu_context.is_surface_configured {
            warn!("Trying to render unconfigured surface");
            return Ok(());
        }

        let mut command_encoder = self
            .gpu_context
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let current_texture = match self.gpu_context.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,

            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,

            wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("Device was lost"),

            wgpu::CurrentSurfaceTexture::Outdated => {
                self.gpu_context.configure_surface();
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

        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..1);

        drop(render_pass);

        self.gpu_context
            .queue
            .submit(std::iter::once(command_encoder.finish()));

        self.gpu_context.queue.present(current_texture);

        Ok(())
    }

    pub fn update(&mut self) {
        self.camera.update(&self.gpu_context.queue);
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, is_pressed: bool) {
        if key == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera.handle_key(key, is_pressed);
        }
    }
}

struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
}

impl Mesh {
    fn new(device: &Device, vertices: &[impl bytemuck::NoUninit], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}
