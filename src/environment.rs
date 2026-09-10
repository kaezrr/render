use std::fs::File;
use std::path::Path;

use wgpu::ShaderModuleDescriptor;

use crate::hdr::HdrLoader;
use crate::hdr::HdrPipeline;
use crate::load_asset_string;
use crate::pipeline;
use crate::texture;
use crate::texture::Texture;

#[derive(Debug)]
pub struct SkyBoxPipeline {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl SkyBoxPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        sky_texture_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let hdr_loader = HdrLoader::new(device)?;

        let sky_texture = hdr_loader.create_cube_texture(
            device,
            queue,
            File::open_buffered(sky_texture_path)?,
            1080,
            Some("Sky Texture"),
        )?;

        let bind_group_layout = texture::util::create_bind_group_layout(
            device,
            1,
            false,
            wgpu::TextureViewDimension::Cube,
            wgpu::SamplerBindingType::NonFiltering,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            Some("environment_layout"),
        );

        let bind_group = texture::util::create_bind_group(
            device,
            &bind_group_layout,
            &[&sky_texture],
            None,
            Some("environment_bind_group"),
        );

        let render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(camera_bind_group_layout), Some(&bind_group_layout)],
                immediate_size: 0,
            });

            let shader = ShaderModuleDescriptor {
                label: Some("sky.wgsl"),
                source: wgpu::ShaderSource::Wgsl(load_asset_string("shaders/sky.wgsl")?.into()),
            };

            pipeline::create_render_pipeline(
                device,
                &layout,
                HdrPipeline::TEXTURE_FORMAT,
                Some(Texture::DEPTH_FORMAT),
                &[],
                shader,
                Some("State::sky_pipeline"),
            )
        };

        Ok(Self {
            bind_group,
            bind_group_layout,
            render_pipeline,
        })
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
