use crate::load_asset_string;
use crate::pipeline;
use crate::texture::Texture;
use crate::texture::{self};

#[derive(Debug)]
pub struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: Texture,
    width: u32,
    height: u32,
    layout: wgpu::BindGroupLayout,
}

impl HdrPipeline {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self> {
        let wgpu::SurfaceConfiguration { width, height, .. } = *config;

        let texture = Texture::create_2d_texture(
            device,
            width,
            height,
            Self::TEXTURE_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("HdrPipeline::texture"),
        );

        let layout =
            texture::create_bind_group_layout(device, 1, false, Some("HdrPipeline::layout"));

        let bind_group = texture::create_bind_group(
            device,
            &layout,
            &[&texture],
            None,
            Some("HdrPipeline::bind_group"),
        );

        let pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("hdr.wgsl"),
                source: wgpu::ShaderSource::Wgsl(load_asset_string("shaders/hdr.wgsl")?.into()),
            };

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Hdr pipeline layout"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });

            pipeline::create_render_pipeline(
                device,
                &pipeline_layout,
                config.format.add_srgb_suffix(),
                None,
                &[],
                shader,
                Some("HdrPipeline::pipeline"),
            )
        };

        Ok(Self {
            pipeline,
            bind_group,
            texture,
            width,
            height,
            layout,
        })
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = Texture::create_2d_texture(
            device,
            width,
            height,
            Self::TEXTURE_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("HdrPipeline::texture"),
        );

        self.bind_group = texture::create_bind_group(
            device,
            &self.layout,
            &[&self.texture],
            None,
            Some("HdrPipeline::bind_group"),
        );

        self.width = width;
        self.height = height;
    }

    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HdrPipeline::process"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
                resolve_target: None,
            })],
            ..Default::default()
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    pub fn view(&self) -> &wgpu::TextureView {
        self.texture.view()
    }
}
