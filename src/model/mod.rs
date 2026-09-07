mod vertex;
use core::ops::Range;

pub use vertex::GpuVertex;
pub use vertex::ModelVertex;

#[expect(unused, reason = "Only using draw model instanced for now")]
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

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

#[derive(Debug)]
pub struct Material {
    #[expect(unused, reason = "Material name is for debug purposes")]
    pub name: String,
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

impl DrawModel for wgpu::RenderPass<'_> {
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
