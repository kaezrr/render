use core::ops::Range;

use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Vec4;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BufferUsages;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::model::Mesh;
use crate::model::Model;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightUniform {
    pub position: Vec4,
    pub color: Vec4,
}

pub trait DrawLight {
    fn draw_light_mesh(
        &mut self,
        mesh: &Mesh,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &Mesh,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    );

    fn draw_light_model(
        &mut self,
        model: &Model,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_model_instanced(
        &mut self,
        model: &Model,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(
                mesh,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
            );
        }
    }
}

impl DrawLight for wgpu::RenderPass<'_> {
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &Mesh,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);

        self.draw_indexed(0..mesh.num_indices, 0, instances);
    }
}

#[derive(Debug)]
pub struct LightBundle {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub uniform: LightUniform,

    buffer: wgpu::Buffer,
}

impl LightBundle {
    pub fn new(device: &wgpu::Device, uniform: LightUniform) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("light_buffer_uniform"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            bind_group,
            bind_group_layout,

            uniform,
            buffer,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}
