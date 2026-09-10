use wgpu::SamplerBindingType;
use wgpu::TextureViewDimension;

use crate::texture::TextureType;

pub fn create_bind_group_layout(
    device: &wgpu::Device,
    texture_count: u32,
    has_uniform_buffer: bool,
    view_dimension: TextureViewDimension,
    sampler_binding_type: SamplerBindingType,
    visibility: wgpu::ShaderStages,
    label: Option<&str>,
) -> wgpu::BindGroupLayout {
    let mut entries = Vec::with_capacity(texture_count as usize * 2);
    let filterable = sampler_binding_type == SamplerBindingType::Filtering;

    for i in 0..texture_count {
        let texture_binding = i * 2;
        let sampler_binding = texture_binding + 1;

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: texture_binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension,
                multisampled: false,
            },
            count: None,
        });

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: sampler_binding,
            visibility,
            ty: wgpu::BindingType::Sampler(sampler_binding_type),
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
    textures: &[&impl TextureType],
    uniform_buffer: Option<&wgpu::Buffer>,
    label: Option<&str>,
) -> wgpu::BindGroup {
    let mut entries = Vec::with_capacity(textures.len() * 2);

    for (i, texture) in textures.iter().enumerate() {
        let texture_binding = i as u32 * 2;
        let sampler_binding = texture_binding + 1;

        entries.push(wgpu::BindGroupEntry {
            binding: texture_binding,
            resource: wgpu::BindingResource::TextureView(texture.view()),
        });

        entries.push(wgpu::BindGroupEntry {
            binding: sampler_binding,
            resource: wgpu::BindingResource::Sampler(texture.sampler()),
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
