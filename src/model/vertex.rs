use bytemuck::NoUninit;
use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Vec2;
use glam::Vec3;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;

pub trait GpuVertex: NoUninit {
    fn desc() -> VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ModelVertex {
    pub position: Vec3,
    pub texture_uv: Vec2,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitanget: Vec3,
}

impl ModelVertex {
    const ATTRIBS: &[VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
        2 => Float32x3,
        3 => Float32x3,
        4 => Float32x3,
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
