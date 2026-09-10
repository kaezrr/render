mod gpu;

use core::time::Duration;
use std::fs::File;
use std::sync::Arc;

use glam::Quat;
use glam::Vec3;
use glam::Vec4Swizzles;
use log::warn;
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
use winit::window::CursorGrabMode;
use winit::window::Window;

use crate::camera::Camera;
use crate::camera::CameraBundle;
use crate::camera::Projection;
use crate::create_asset_path;
use crate::hdr::HdrLoader;
use crate::hdr::HdrPipeline;
use crate::instance::Instance;
use crate::instance::InstanceBundle;
use crate::instance::InstanceRaw;
use crate::light::DrawLight;
use crate::light::LightBundle;
use crate::light::LightUniform;
use crate::load_asset_string;
use crate::model::DrawModel;
use crate::model::GpuVertex;
use crate::model::Material;
use crate::model::Model;
use crate::model::ModelVertex;
use crate::model::PropertiesUniform;
use crate::parser::load_model_from_obj;
use crate::pipeline;
use crate::state::gpu::GpuContext;
use crate::texture;
use crate::texture::Texture;
use crate::texture::TextureType;

#[derive(Debug)]
pub struct State<'a> {
    window: Arc<Window>,
    gpu_context: GpuContext<'a>,

    obj_model: Model,
    depth_texture: Texture,
    instance_bundle: InstanceBundle,
    default_material: Material,

    camera: CameraBundle,
    light: LightBundle,

    hdr: HdrPipeline,
    render_pipeline: RenderPipeline,
    light_render_pipeline: RenderPipeline,
    sky_render_pipeline: RenderPipeline,

    environment_bind_group: wgpu::BindGroup,

    cursor_grabbed: bool,
}

impl State<'_> {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu_context = GpuContext::new(window.clone()).await?;

        let camera = create_camera_bundle(&gpu_context.device, &gpu_context.config);

        let light = create_light_bundle(&gpu_context.device);

        let material_bind_group_layout = texture::util::create_bind_group_layout(
            &gpu_context.device,
            2,    // Diffuse and Normal Texture
            true, // with uniform buffer for properties
            wgpu::TextureViewDimension::D2,
            wgpu::SamplerBindingType::Filtering,
            wgpu::ShaderStages::FRAGMENT,
            Some("Material Bind Group Layout"),
        );

        let default_material = Material::new(
            &gpu_context.device,
            &gpu_context.queue,
            "Default Material".to_string(),
            PropertiesUniform::DEFAULT_MAT,
            &material_bind_group_layout,
            None,
            None,
        );

        let obj_model = load_model_from_obj(
            &gpu_context.device,
            &gpu_context.queue,
            &material_bind_group_layout,
            "models/skull/Skull.obj",
        )?;

        let light_render_pipeline = {
            let layout = gpu_context
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Light Pipeline Layout"),
                    bind_group_layouts: &[
                        Some(&camera.bind_group_layout),
                        Some(&light.bind_group_layout),
                    ],
                    immediate_size: 0,
                });

            let shader = ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(load_asset_string("shaders/light.wgsl")?.into()),
            };

            pipeline::create_render_pipeline(
                &gpu_context.device,
                &layout,
                HdrPipeline::TEXTURE_FORMAT,
                Some(Texture::DEPTH_FORMAT),
                &[Some(ModelVertex::desc())],
                shader,
                Some("Light Render Pipeline"),
            )
        };

        let instance_bundle = InstanceBundle::new(
            &gpu_context.device,
            vec![Instance {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            }],
        );

        let depth_texture = Texture::create_depth_texture(&gpu_context.device, &gpu_context.config);

        let hdr = HdrPipeline::new(&gpu_context.device, &gpu_context.config)?;

        let hdr_loader = HdrLoader::new(&gpu_context.device)?;

        let sky_texture = hdr_loader.create_cube_texture(
            &gpu_context.device,
            &gpu_context.queue,
            File::open_buffered(create_asset_path("pure-sky.hdr"))?,
            1080,
            Some("Sky Texture"),
        )?;

        let env_bind_group_layout = texture::util::create_bind_group_layout(
            &gpu_context.device,
            1,
            false,
            wgpu::TextureViewDimension::Cube,
            wgpu::SamplerBindingType::NonFiltering,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            Some("environment_layout"),
        );

        let environment_bind_group = texture::util::create_bind_group(
            &gpu_context.device,
            &env_bind_group_layout,
            &[&sky_texture],
            None,
            Some("environment_bind_group"),
        );

        let sky_render_pipeline = {
            let layout =
                gpu_context
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[
                            Some(&camera.bind_group_layout),
                            Some(&env_bind_group_layout),
                        ],
                        immediate_size: 0,
                    });

            let shader = ShaderModuleDescriptor {
                label: Some("sky.wgsl"),
                source: wgpu::ShaderSource::Wgsl(load_asset_string("shaders/sky.wgsl")?.into()),
            };

            pipeline::create_render_pipeline(
                &gpu_context.device,
                &layout,
                HdrPipeline::TEXTURE_FORMAT,
                Some(Texture::DEPTH_FORMAT),
                &[],
                shader,
                Some("State::sky_pipeline"),
            )
        };

        let render_pipeline = {
            let layout = gpu_context
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Model Pipeline Layout"),
                    immediate_size: 0,
                    bind_group_layouts: &[
                        Some(&material_bind_group_layout),
                        Some(&camera.bind_group_layout),
                        Some(&light.bind_group_layout),
                        Some(&env_bind_group_layout),
                    ],
                });

            let shader = ShaderModuleDescriptor {
                label: Some("Model Shader"),
                source: wgpu::ShaderSource::Wgsl(load_asset_string("shaders/model.wgsl")?.into()),
            };

            pipeline::create_render_pipeline(
                &gpu_context.device,
                &layout,
                HdrPipeline::TEXTURE_FORMAT,
                Some(Texture::DEPTH_FORMAT),
                &[Some(ModelVertex::desc()), Some(InstanceRaw::desc())],
                shader,
                Some("Model Render Pipeline"),
            )
        };
        Ok(Self {
            window,
            gpu_context,

            obj_model,
            depth_texture,
            instance_bundle,
            default_material,

            camera,
            light,

            hdr,
            sky_render_pipeline,
            render_pipeline,
            light_render_pipeline,

            environment_bind_group,

            cursor_grabbed: false,
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
                view: self.hdr.view(),
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
                view: self.depth_texture.view(),
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

        render_pass.set_pipeline(&self.sky_render_pipeline);
        render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &self.environment_bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        render_pass.set_pipeline(&self.light_render_pipeline);
        // render_pass.draw_light_model(
        //     &self.obj_model,
        //     &self.camera.bind_group,
        //     &self.light.bind_group,
        // );

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(1, self.instance_bundle.buffer.slice(..));

        render_pass.draw_model_instanced(
            &self.obj_model,
            &self.default_material,
            0..self.instance_bundle.instances.len() as u32,
            &self.camera.bind_group,
            &self.light.bind_group,
            &self.environment_bind_group,
        );

        drop(render_pass);

        self.hdr.process(&mut command_encoder, &texture_view);

        self.gpu_context
            .queue
            .submit(core::iter::once(command_encoder.finish()));

        self.gpu_context.queue.present(current_texture);

        Ok(())
    }

    pub fn update(&mut self, dt: Duration) {
        let old_position = self.light.uniform.position;
        let light_rotation_angle = f32::to_radians(50.0) * dt.as_secs_f32();
        let light_rotation = Quat::from_rotation_y(light_rotation_angle);

        self.light.uniform.position = (light_rotation * old_position.xyz()).to_homogeneous();

        self.light.update(&self.gpu_context.queue);
        self.camera.update(&self.gpu_context.queue, dt);

        self.instance_bundle.update(&self.gpu_context.queue);
    }

    pub fn process_keyboard(&mut self, key: KeyCode, is_pressed: bool) {
        self.camera.controller.process_keyboard(key, is_pressed);
    }

    pub fn process_mouse_delta(&mut self, dx: f64, dy: f64) {
        if !self.cursor_grabbed {
            return;
        }
        self.camera.controller.process_mouse_delta(dx, dy);
    }

    pub fn process_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        if !self.cursor_grabbed {
            return;
        }
        self.camera.controller.process_mouse_scroll(delta);
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.camera.projection.resize(width, height);
        self.gpu_context.resize_surface(width, height);
        self.hdr.resize(&self.gpu_context.device, width, height);
        self.depth_texture =
            Texture::create_depth_texture(&self.gpu_context.device, &self.gpu_context.config);
    }

    pub fn capture_mouse(&mut self) -> anyhow::Result<()> {
        self.window.set_cursor_grab(CursorGrabMode::Locked)?;
        self.window.set_cursor_visible(false);
        self.cursor_grabbed = true;
        Ok(())
    }

    pub fn release_mouse(&mut self) -> anyhow::Result<()> {
        self.window.set_cursor_grab(CursorGrabMode::None)?;
        self.window.set_cursor_visible(true);
        self.cursor_grabbed = false;
        Ok(())
    }
}

fn create_camera_bundle(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> CameraBundle {
    CameraBundle::new(
        device,
        Camera::new((0.0, 1.5, 3.0), -90.0, -12.0),
        Projection::new(config.width, config.height, 60.0, 0.1, 100.0),
        4.0,
        0.4,
    )
}

fn create_light_bundle(device: &wgpu::Device) -> LightBundle {
    LightBundle::new(
        device,
        LightUniform {
            position: (5.0, 5.0, 5.0, 1.0).into(),
            color: (1.0, 1.0, 1.0, 1.0).into(),
        },
    )
}
