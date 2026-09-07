use core::ops::Range;

use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Vec4;

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
