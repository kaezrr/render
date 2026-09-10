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
use wgpu::PolygonMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::ShaderModuleDescriptor;
use wgpu::TextureFormat;
use wgpu::VertexState;

pub fn create_render_pipeline(
    device: &Device,
    layout: &wgpu::PipelineLayout,
    color_format: TextureFormat,
    depth_stencil_format: Option<TextureFormat>,
    vertex_layouts: &[Option<wgpu::VertexBufferLayout>],
    shader: ShaderModuleDescriptor,
    label: Option<&str>,
) -> RenderPipeline {
    let shader_module = device.create_shader_module(shader);

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        vertex: VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: vertex_layouts,
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
            depth_compare: Some(CompareFunction::LessEqual),
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
                format: color_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),

        multiview_mask: None,
        cache: None,
    })
}
