use core::mem::size_of;

use image::ColorType;
use image::ImageDecoder;
use image::codecs::hdr::HdrDecoder;
use wgpu::wgt::CommandEncoderDescriptor;

use crate::load_asset_string;
use crate::pipeline;
use crate::texture;
use crate::texture::Texture;
use crate::texture::TextureType;

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

        let layout = texture::util::create_bind_group_layout(
            device,
            1,
            false,
            wgpu::TextureViewDimension::D2,
            wgpu::SamplerBindingType::Filtering,
            wgpu::ShaderStages::FRAGMENT,
            Some("HdrPipeline::layout"),
        );

        let bind_group = texture::util::create_bind_group(
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

        self.bind_group = texture::util::create_bind_group(
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

pub struct HdrLoader {
    equirect_layout: wgpu::BindGroupLayout,
    equirect_to_cubemap: wgpu::ComputePipeline,
}

impl HdrLoader {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub fn new(device: &wgpu::Device) -> anyhow::Result<Self> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("equirectanglar.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                load_asset_string("shaders/equirectangular.wgsl")?.into(),
            ),
        });

        let equirect_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("HdrLoader::equirect_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: Self::TEXTURE_FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&equirect_layout)],
            immediate_size: 0,
        });

        let equirect_to_cubemap =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("HdrPipeline::equirect_to_cubemap"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some("compute_equirect_to_cubemap"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        Ok(Self {
            equirect_layout,
            equirect_to_cubemap,
        })
    }

    pub fn create_cube_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        hdr_data: impl std::io::Read,
        dst_size: u32,
        label: Option<&str>,
    ) -> anyhow::Result<texture::CubeTexture> {
        let hdr_decoder = HdrDecoder::new(hdr_data)?;
        let metadata = hdr_decoder.metadata();
        let total_bytes = hdr_decoder.total_bytes() as usize;

        assert_eq!(hdr_decoder.color_type(), ColorType::Rgb32F);

        let mut pixels: Vec<[f32; 3]> = vec![[0.; 3]; total_bytes / size_of::<[f32; 3]>()];
        hdr_decoder.read_image(bytemuck::cast_slice_mut(&mut pixels))?;

        let pixels = pixels
            .into_iter()
            .map(|[r, g, b]| [r, g, b, 1.0])
            .collect::<Vec<_>>();

        let src = texture::Texture::create_2d_texture(
            device,
            metadata.width,
            metadata.height,
            Self::TEXTURE_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            wgpu::FilterMode::Nearest,
            None,
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(src.size().width * size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(src.size().height),
            },
            src.size(),
        );

        let dst = texture::CubeTexture::create_2d(
            device,
            dst_size,
            dst_size,
            Self::TEXTURE_FORMAT,
            1,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Nearest,
            label,
        );

        let dst_view = dst.texture().create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &self.equirect_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label,
            timestamp_writes: None,
        });

        let num_workgroups = dst_size.div_ceil(16);
        compute_pass.set_pipeline(&self.equirect_to_cubemap);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

        drop(compute_pass);

        queue.submit([encoder.finish()]);

        Ok(dst)
    }
}
