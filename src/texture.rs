use image::GenericImageView;
use wgpu::Device;
use wgpu::TextureUsages;
use wgpu::util::DeviceExt;
use wgpu::wgt::SamplerDescriptor;
use wgpu::wgt::TextureDescriptor;

#[derive(Debug)]
pub struct Texture {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
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
        device: &Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: config.width.max(1),
                height: config.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
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

    pub fn from_solid_color(
        device: &Device,
        queue: &wgpu::Queue,
        color: [f32; 3],
        label: Option<&str>,
    ) -> Self {
        let pixel = image::Rgb(color);
        let image = image::Rgb32FImage::from_pixel(1, 1, pixel);

        Self::from_image(
            device,
            queue,
            &image::DynamicImage::ImageRgb32F(image),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            label,
        )
    }

    pub fn create_solid_normal(device: &Device, queue: &wgpu::Queue, label: Option<&str>) -> Self {
        let image = image::RgbImage::from_pixel(1, 1, image::Rgb([128, 128, 255]));

        Self::from_image(
            device,
            queue,
            &image::DynamicImage::ImageRgb8(image),
            wgpu::TextureFormat::Rgba8Unorm,
            label,
        )
    }
}

pub fn create_bind_group_layout(
    device: &wgpu::Device,
    label: &str,
    texture_count: usize,
) -> wgpu::BindGroupLayout {
    let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::with_capacity(texture_count * 2);

    for i in 0..texture_count {
        let texture_binding = i as u32 * 2;
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

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &entries,
    })
}

pub fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    textures: &[Texture],
    label: Option<&str>,
) -> wgpu::BindGroup {
    let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();

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

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout,
        entries: &entries,
    })
}
