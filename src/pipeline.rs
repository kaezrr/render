use wgpu::BindGroupLayout;
use wgpu::BlendState;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CompareFunction;
use wgpu::DepthStencilState;
use wgpu::Device;
use wgpu::Face;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::MultisampleState;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::ShaderModuleDescriptor;
use wgpu::TextureFormat;
use wgpu::VertexState;

use crate::instance::InstanceRaw;
use crate::vertex::GpuVertex;

pub fn create_render_pipeline<V: GpuVertex>(
    device: &Device,
    label: &str,
    shader_source: &str,
    surface_format: TextureFormat,
    bind_group_layouts: &[Option<&BindGroupLayout>],
    depth_stencil_format: Option<TextureFormat>,
) -> RenderPipeline {
    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("render_pipeline_layout"),
        bind_group_layouts,
        immediate_size: 0,
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("render_pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(V::desc()), Some(InstanceRaw::desc())],
        },

        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },

        depth_stencil: depth_stencil_format.map(|format| DepthStencilState {
            format,
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),

        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },

        fragment: Some(FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format: surface_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),

        multiview_mask: None,
        cache: None,
    })
}
