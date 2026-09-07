use bytemuck::NoUninit;
use bytemuck::Pod;
use bytemuck::Zeroable;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;

pub trait GpuVertex: NoUninit {
    fn desc() -> VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub texture_uv: [f32; 2],
    pub normal: [f32; 3],
}

impl ModelVertex {
    const ATTRIBS: &[VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
        2 => Float32x3,
    ];
}

impl GpuVertex for ModelVertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: core::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}
