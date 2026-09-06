use core::ops::Range;

use bytemuck::NoUninit;
use bytemuck::Pod;
use bytemuck::Zeroable;
use wgpu::RenderPass;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;

use crate::texture::Texture;

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

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

#[derive(Debug)]
pub struct Material {
    #[expect(unused, reason = "Material name is for debug purposes")]
    pub name: String,
    pub _diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct Mesh {
    #[expect(unused, reason = "Mesh name is for debug purposes")]
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub material_id: Option<usize>,
}

pub trait DrawModel {
    fn draw_mesh(&mut self, mesh: &Mesh, material: &Material, camera_bind_group: &wgpu::BindGroup);

    fn draw_mesh_instanced(
        &mut self,
        mesh: &Mesh,
        material: &Material,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
    );

    fn draw_model(
        &mut self,
        model: &Model,
        default_material: &Material,
        camera_bind_group: &wgpu::BindGroup,
    );

    fn draw_model_instanced(
        &mut self,
        model: &Model,
        default_material: &Material,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
    );
}

impl DrawModel for RenderPass<'_> {
    fn draw_mesh(&mut self, mesh: &Mesh, material: &Material, camera_bind_group: &wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &Mesh,
        material: &Material,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, &material.bind_group, &[]);

        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.draw_indexed(0..mesh.num_indices, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &Model,
        default_material: &Material,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, default_material, 0..1, camera_bind_group);
    }

    fn draw_model_instanced(
        &mut self,
        model: &Model,
        default_material: &Material,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = mesh
                .material_id
                .map_or(default_material, |i| &model.materials[i]);

            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group);
        }
    }
}
