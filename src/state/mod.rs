mod gpu;

use std::sync::Arc;

use glam::Quat;
use glam::Vec3;
use log::warn;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::Color;
use wgpu::Operations;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::wgt::CommandEncoderDescriptor;
use wgpu::wgt::TextureViewDescriptor;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::camera::Camera;
use crate::camera::CameraBundle;
use crate::instance::Instance;
use crate::instance::InstanceBundle;
use crate::load_asset_bytes;
use crate::load_asset_string;
use crate::model::DrawModel;
use crate::model::Model;
use crate::model::ModelVertex;
use crate::parser::load_model_from_obj;
use crate::pipeline;
use crate::state::gpu::GpuContext;
use crate::texture::Texture;
use crate::texture::TextureBundle;

#[derive(Debug)]
pub struct State<'a> {
    window: Arc<Window>,
    gpu_context: GpuContext<'a>,

    obj_model: Model,
    render_pipeline: RenderPipeline,
    depth_texture: Texture,

    instance_bundle: InstanceBundle,
    diffuse_texture: TextureBundle,

    camera: CameraBundle,
}

impl State<'_> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu_context = GpuContext::new(window.clone()).await?;

        let camera = CameraBundle::new(
            &gpu_context.device,
            Camera {
                eye: (0.0, 1.0, 2.0).into(),
                target: (0.0, 0.0, 0.0).into(),
                up: glam::Vec3::Y,
                aspect_ratio: gpu_context.config.width as f32 / gpu_context.config.height as f32,
                vertical_fov: f32::to_radians(45.0),
                znear: 0.1,
                zfar: 100.0,
            },
            5.0,
        );

        let texture_bind_group_layout =
            gpu_context
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("texture_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let diffuse_texture = Texture::from_bytes(
            &gpu_context.device,
            &gpu_context.queue,
            &load_asset_bytes("happy-tree.png")?,
            "happy_tree_texture",
        )?
        .with_bind_group(&gpu_context.device, &texture_bind_group_layout);

        let render_pipeline = pipeline::create_render_pipeline::<ModelVertex>(
            &gpu_context.device,
            "colored",
            &load_asset_string("shaders/shader.wgsl")?,
            gpu_context.config.format,
            &[
                Some(&camera.bind_group_layout),
                Some(&texture_bind_group_layout),
            ],
            Some(Texture::DEPTH_FORMAT),
        );

        const NUM_INSTANCES_PER_ROW: u32 = 10;

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    const SPACE_BETWEEN: f32 = 3.0;

                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = Vec3::new(x, 0.0, z);

                    let rotation = if position == Vec3::ZERO {
                        Quat::from_rotation_z(0.0)
                    } else {
                        Quat::from_rotation_z(45.0f32.to_radians())
                    };

                    Instance {
                        position,
                        rotation,
                        scale: Vec3::ONE * 0.6,
                    }
                })
            })
            .collect::<Vec<_>>();

        let instance_bundle = InstanceBundle::new(&gpu_context.device, instances);

        let depth_texture = Texture::create_depth_texture(
            &gpu_context.device,
            &gpu_context.config,
            "depth_texture",
        );

        let obj_model = load_model_from_obj(
            &gpu_context.device,
            &gpu_context.queue,
            &texture_bind_group_layout,
            "models/cube/cube.obj",
        )?;

        Ok(Self {
            window,
            gpu_context,

            obj_model,
            render_pipeline,
            depth_texture,

            instance_bundle,
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
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,

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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &self.diffuse_texture.bind_group, &[]);

        render_pass.set_vertex_buffer(1, self.instance_bundle.buffer.slice(..));
        render_pass.draw_mesh_instanced(
            &self.obj_model.meshes[0],
            0..self.instance_bundle.instances.len() as u32,
        );

        drop(render_pass);

        self.gpu_context
            .queue
            .submit(core::iter::once(command_encoder.finish()));

        self.gpu_context.queue.present(current_texture);

        Ok(())
    }

    pub fn update(&mut self, dt: f32) {
        self.camera.update(&self.gpu_context.queue, dt);

        let rotation_speed = f32::to_radians(20.0) * dt;
        for (i, instance) in self.instance_bundle.instances.iter_mut().enumerate() {
            let rotation = if i % 2 == 0 {
                Quat::from_rotation_x(rotation_speed)
            } else {
                Quat::from_rotation_y(rotation_speed)
            };

            instance.rotation *= rotation;
        }

        self.instance_bundle.update(&self.gpu_context.queue);
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, is_pressed: bool) {
        if key == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera.handle_key(key, is_pressed);
        }
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.gpu_context.resize_surface(width, height);
    }
}
