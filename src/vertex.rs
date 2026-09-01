#![allow(unused)]

use std::fmt::Debug;

use bytemuck::NoUninit;
use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Mat3A;
use glam::Mat4;
use glam::camera;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;

pub trait GpuVertex: NoUninit {
    fn desc() -> VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];
}

impl GpuVertex for Vertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub texture_coordinates: [f32; 2],
}

impl TexturedVertex {
    const ATTRIBS: [VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
    ];
}

impl GpuVertex for TexturedVertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
