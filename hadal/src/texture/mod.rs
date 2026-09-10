pub mod util;

use image::GenericImageView;
use wgpu::TextureUsages;
use wgpu::util::DeviceExt;
use wgpu::wgt::SamplerDescriptor;
use wgpu::wgt::TextureDescriptor;

pub trait TextureType {
    fn texture(&self) -> &wgpu::Texture;
    fn view(&self) -> &wgpu::TextureView;
    fn sampler(&self) -> &wgpu::Sampler;
}

#[derive(Debug)]
pub struct Texture {
    raw: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    size: wgpu::Extent3d,
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

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let raw = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label,
                size,
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

        let view = raw.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            raw,
            view,
            sampler,
            size,
        }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let raw = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = raw.create_view(&wgpu::wgt::TextureViewDescriptor::default());

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

        Self {
            raw,
            view,
            sampler,
            size,
        }
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
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let raw = device.create_texture(&TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = raw.create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            raw,
            view,
            sampler,
            size,
        }
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

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}

#[derive(Debug)]
pub struct CubeTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl CubeTexture {
    #[expect(clippy::too_many_arguments, reason = "This is a complicated function")]
    pub fn create_2d(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        mip_level_count: u32,
        usage: wgpu::TextureUsages,
        mag_filter: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::wgt::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

impl TextureType for Texture {
    fn texture(&self) -> &wgpu::Texture {
        &self.raw
    }

    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

impl TextureType for CubeTexture {
    fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
