mod gpu;

use core::time::Duration;
use std::sync::Arc;

use glam::Quat;
use glam::Vec3;
use log::warn;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::Color;
use wgpu::Operations;
use wgpu::PipelineLayoutDescriptor;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::ShaderModuleDescriptor;
use wgpu::wgt::CommandEncoderDescriptor;
use wgpu::wgt::TextureViewDescriptor;
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::camera::Camera;
use crate::camera::CameraBundle;
use crate::camera::Projection;
use crate::instance::Instance;
use crate::instance::InstanceBundle;
use crate::instance::InstanceRaw;
use crate::load_asset_string;
use crate::model::DrawModel;
use crate::model::GpuVertex;
use crate::model::Material;
use crate::model::Model;
use crate::model::ModelVertex;
use crate::parser::load_model_from_obj;
use crate::pipeline;
use crate::state::gpu::GpuContext;
use crate::texture::Texture;

#[derive(Debug)]
pub struct State<'a> {
    window: Arc<Window>,
    gpu_context: GpuContext<'a>,

    obj_model: Model,
    depth_texture: Texture,
    instance_bundle: InstanceBundle,
    default_material: Material,

    render_pipeline: RenderPipeline,
    camera: CameraBundle,
}

impl State<'_> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu_context = GpuContext::new(window.clone()).await?;

        let camera = {
            let config = &gpu_context.config;
            CameraBundle::new(
                &gpu_context.device,
                Camera::new((0.0, 5.0, 10.0), -90.0, -20.0),
                Projection::new(config.width, config.height, 45.0, 0.1, 100.0),
                4.0,
                0.4,
            )
        };

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

        let render_pipeline = {
            let shader = ShaderModuleDescriptor {
                label: Some("model.wgsl"),
                source: wgpu::ShaderSource::Wgsl(load_asset_string("shaders/model.wgsl")?.into()),
            };

            let render_pipeline_layout =
                gpu_context
                    .device
                    .create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("render_pipeline_layout"),
                        immediate_size: 0,
                        bind_group_layouts: &[
                            Some(&camera.bind_group_layout),
                            Some(&texture_bind_group_layout),
                        ],
                    });

            pipeline::create_render_pipeline(
                &gpu_context.device,
                &render_pipeline_layout,
                gpu_context.config.format,
                Some(Texture::DEPTH_FORMAT),
                &[Some(ModelVertex::desc()), Some(InstanceRaw::desc())],
                shader,
            )
        };

        let instance_bundle = {
            const SPACE_BETWEEN: f32 = 3.0;
            const NUM_INSTANCES_PER_ROW: u32 = 10;
            let instances = (0..NUM_INSTANCES_PER_ROW)
                .flat_map(|z| {
                    (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                        let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                        let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                        let position = Vec3 { x, y: 0.0, z };

                        let rotation = if position == Vec3::ZERO {
                            Quat::from_axis_angle(Vec3::Z, 0.0f32.to_radians())
                        } else {
                            Quat::from_axis_angle(position.normalize(), 45.0f32.to_radians())
                        };

                        Instance {
                            position,
                            rotation,
                            scale: Vec3::ONE,
                        }
                    })
                })
                .collect();
            InstanceBundle::new(&gpu_context.device, instances)
        };

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

        let default_material = create_default_material(&gpu_context, &texture_bind_group_layout);

        Ok(Self {
            window,
            gpu_context,

            obj_model,
            depth_texture,
            instance_bundle,
            default_material,

            render_pipeline,
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
        render_pass.set_vertex_buffer(1, self.instance_bundle.buffer.slice(..));

        render_pass.draw_model_instanced(
            &self.obj_model,
            &self.default_material,
            0..self.instance_bundle.instances.len() as u32,
            &self.camera.bind_group,
        );

        drop(render_pass);

        self.gpu_context
            .queue
            .submit(core::iter::once(command_encoder.finish()));

        self.gpu_context.queue.present(current_texture);

        Ok(())
    }

    pub fn update(&mut self, dt: Duration) {
        self.camera.update(&self.gpu_context.queue, dt);

        let angle = f32::to_radians(10.0) * dt.as_secs_f32();
        let axis = Vec3::new(1.0, 1.0, 0.0).normalize();
        let rotation = Quat::from_axis_angle(axis, angle);

        for instance in &mut self.instance_bundle.instances {
            instance.rotation *= rotation;
        }

        self.instance_bundle.update(&self.gpu_context.queue);
    }

    pub fn process_keyboard(&mut self, key: KeyCode, is_pressed: bool) {
        self.camera.controller.process_keyboard(key, is_pressed);
    }

    pub fn process_mouse_delta(&mut self, dx: f64, dy: f64) {
        self.camera.controller.process_mouse_delta(dx, dy);
    }

    pub fn process_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        self.camera.controller.process_mouse_scroll(delta);
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.camera.projection.resize(width, height);
        self.gpu_context.resize_surface(width, height);
        self.depth_texture = Texture::create_depth_texture(
            &self.gpu_context.device,
            &self.gpu_context.config,
            "depth_texture",
        );
    }
}

fn create_default_material(gpu_context: &GpuContext, layout: &wgpu::BindGroupLayout) -> Material {
    let diffuse_texture = Texture::from_solid_color(
        &gpu_context.device,
        &gpu_context.queue,
        [1.0, 0.0, 1.0],
        Some("default render texture"),
    );

    let bind_group = diffuse_texture.create_bind_group(
        &gpu_context.device,
        layout,
        Some("Bind Group: default_material"),
    );

    Material {
        name: "default render material".to_string(),
        bind_group,
    }
}
