mod gpu;

use std::sync::Arc;

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

use crate::asset_str;
use crate::camera::Camera;
use crate::camera::CameraBundle;
use crate::consts::CUBE_INDICES;
use crate::consts::CUBE_VERTICES;
use crate::consts::INDICES;
use crate::consts::TEXTURED_VERTICES;
use crate::consts::TRIANGLE_INDICES;
use crate::consts::TRIANGLE_VERTICES;
use crate::consts::VERTICES;
use crate::mesh::Mesh;
use crate::pipeline;
use crate::state::gpu::GpuContext;
use crate::vertex::Vertex;

pub struct State<'a> {
    pub window: Arc<Window>,
    pub gpu_context: GpuContext<'a>,

    render_pipeline: RenderPipeline,
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
                vertical_fov: 45.0f32.to_radians(),
                znear: 0.1,
                zfar: 100.0,
            },
            0.005,
        );

        let render_pipeline = pipeline::create_render_pipeline::<Vertex>(
            &gpu_context.device,
            "colored",
            asset_str!("shaders/colored.wgsl"),
            gpu_context.config.format,
            &[Some(&camera.bind_group_layout)],
        );

        let mesh = Mesh::new(&gpu_context.device, CUBE_VERTICES, CUBE_INDICES);

        Ok(Self {
            window,
            gpu_context,

            render_pipeline,
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

        render_pass.set_bind_group(0, &self.camera.bind_group, &[]);

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
