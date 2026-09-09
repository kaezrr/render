use image::GenericImageView;
use wgpu::TextureUsages;
use wgpu::util::DeviceExt;
use wgpu::wgt::SamplerDescriptor;
use wgpu::wgt::TextureDescriptor;

#[derive(Debug)]
pub struct Texture {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        format: wgpu::TextureFormat,
        label: Option<&str>,
    ) -> anyhow::Result<Self> {
        let image = image::load_from_memory(bytes)?;

        Ok(Self::from_image(device, queue, &image, format, label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
        format: wgpu::TextureFormat,
        label: Option<&str>,
    ) -> Self {
        let dimensions = image.dimensions();

        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::wgt::TextureDataOrder::LayerMajor,
            &image.to_rgba8(),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self { view, sampler }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width.max(1),
                height: config.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        Self { view, sampler }
    }

    pub fn create_2d_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        filter_mode: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self { view, sampler }
    }

    pub fn default_normal(device: &wgpu::Device, queue: &wgpu::Queue, label: Option<&str>) -> Self {
        let image = image::RgbImage::from_pixel(1, 1, image::Rgb([128, 128, 255]));

        Self::from_image(
            device,
            queue,
            &image::DynamicImage::ImageRgb8(image),
            wgpu::TextureFormat::Rgba8Unorm,
            label,
        )
    }

    pub fn default_diffuse(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
    ) -> Self {
        let image = image::RgbImage::from_pixel(1, 1, image::Rgb([255, 255, 255]));

        Self::from_image(
            device,
            queue,
            &image::DynamicImage::ImageRgb8(image),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            label,
        )
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

pub fn create_bind_group_layout(
    device: &wgpu::Device,
    texture_count: u32,
    has_uniform_buffer: bool,
    label: Option<&str>,
) -> wgpu::BindGroupLayout {
    let mut entries = Vec::with_capacity(texture_count as usize * 2);
    for i in 0..texture_count {
        let texture_binding = i * 2;
        let sampler_binding = texture_binding + 1;

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: texture_binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: sampler_binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
    }

    if has_uniform_buffer {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: texture_count * 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &entries,
    })
}

pub fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    textures: &[&Texture],
    uniform_buffer: Option<&wgpu::Buffer>,
    label: Option<&str>,
) -> wgpu::BindGroup {
    let mut entries = Vec::with_capacity(textures.len() * 2);

    for (i, texture) in textures.iter().enumerate() {
        let texture_binding = i as u32 * 2;
        let sampler_binding = texture_binding + 1;

        entries.push(wgpu::BindGroupEntry {
            binding: texture_binding,
            resource: wgpu::BindingResource::TextureView(&texture.view),
        });

        entries.push(wgpu::BindGroupEntry {
            binding: sampler_binding,
            resource: wgpu::BindingResource::Sampler(&texture.sampler),
        });
    }

    if let Some(buffer) = uniform_buffer {
        entries.push(wgpu::BindGroupEntry {
            binding: textures.len() as u32 * 2,
            resource: buffer.as_entire_binding(),
        });
    }

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout,
        entries: &entries,
    })
}
