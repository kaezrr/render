mod gpu;

use std::sync::Arc;

use glam::Quat;
use glam::Vec3;
use log::warn;
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

use crate::asset_bytes;
use crate::asset_str;
use crate::camera::Camera;
use crate::camera::CameraBundle;
use crate::consts::INSTANCE_DISPLACEMENT;
use crate::consts::NUM_INSTANCES_PER_ROW;
use crate::consts::TEXTURED_CUBE_INDICES;
use crate::consts::TEXTURED_CUBE_VERTICES;
use crate::instance::Instance;
use crate::instance::InstanceBundle;
use crate::mesh::Mesh;
use crate::pipeline;
use crate::state::gpu::GpuContext;
use crate::texture;
use crate::texture::TextureBundle;
use crate::vertex::TexturedVertex;

#[derive(Debug)]
pub struct State<'a> {
    pub window: Arc<Window>,
    pub gpu_context: GpuContext<'a>,

    render_pipeline: RenderPipeline,
    diffuse_texture: TextureBundle,
    instance_bundle: InstanceBundle,

    mesh: Mesh,
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

        let texture_bind_group_layout = texture::texture_bind_group_layout(&gpu_context.device);

        let diffuse_texture = TextureBundle::from_bytes(
            &gpu_context.device,
            &gpu_context.queue,
            asset_bytes!("happy-tree.png"),
            "happy_tree_texture",
            &texture_bind_group_layout,
        )?;

        let render_pipeline = pipeline::create_render_pipeline::<TexturedVertex>(
            &gpu_context.device,
            "colored",
            asset_str!("shaders/texture.wgsl"),
            gpu_context.config.format,
            &[
                Some(&camera.bind_group_layout),
                Some(&texture_bind_group_layout),
            ],
        );

        let mesh = Mesh::new(
            &gpu_context.device,
            TEXTURED_CUBE_VERTICES,
            TEXTURED_CUBE_INDICES,
        );

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = Vec3::new(x as f32, 0.0, z as f32) - INSTANCE_DISPLACEMENT;
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

        Ok(Self {
            window,
            gpu_context,

            render_pipeline,
            diffuse_texture,
            instance_bundle,

            mesh,
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
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &self.diffuse_texture.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_bundle.buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(
            0..self.mesh.num_indices,
            0,
            0..self.instance_bundle.instances.len() as _,
        );

        drop(render_pass);

        self.gpu_context
            .queue
            .submit(std::iter::once(command_encoder.finish()));

        self.gpu_context.queue.present(current_texture);

        Ok(())
    }

    pub fn update(&mut self, dt: f32) {
        self.camera.update(&self.gpu_context.queue, dt);

        let rotation_speed = f32::to_radians(10.0) * dt;
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
}
