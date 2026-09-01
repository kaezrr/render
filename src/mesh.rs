use wgpu::Buffer;
use wgpu::BufferUsages;
use wgpu::Device;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::vertex::GpuVertex;

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
}

impl Mesh {
    pub fn new(device: &Device, vertices: &[impl GpuVertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}
