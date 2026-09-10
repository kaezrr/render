mod vertex;
use core::ops::Range;

use bytemuck::Pod;
use bytemuck::Zeroable;
use glam::Vec4;
pub use vertex::GpuVertex;
pub use vertex::ModelVertex;
use wgpu::util::DeviceExt;

use crate::texture;

#[expect(unused, reason = "Only using draw model instanced for now")]
pub trait DrawModel {
    fn draw_mesh(
        &mut self,
        mesh: &Mesh,
        material: &Material,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
        env_bind_group: &wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(
            mesh,
            material,
            0..1,
            camera_bind_group,
            light_bind_group,
            env_bind_group,
        );
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &Mesh,
        material: &Material,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
        env_bind_group: &wgpu::BindGroup,
    );

    fn draw_model(
        &mut self,
        model: &Model,
        default_material: &Material,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
        env_bind_group: &wgpu::BindGroup,
    ) {
        self.draw_model_instanced(
            model,
            default_material,
            0..1,
            camera_bind_group,
            light_bind_group,
            env_bind_group,
        );
    }

    fn draw_model_instanced(
        &mut self,
        model: &Model,
        default_material: &Material,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
        env_bind_group: &wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = mesh
                .material_id
                .map_or(default_material, |i| &model.materials[i]);

            self.draw_mesh_instanced(
                mesh,
                material,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
                env_bind_group,
            );
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
    pub bind_group: wgpu::BindGroup,

    #[expect(unused, reason = "Material properites are constant after creation")]
    properties: PropertiesUniform,
    #[expect(unused, reason = "Material properites are constant after creation")]
    buffer: wgpu::Buffer,
}

impl Material {
    /// First texture is diffuse texture, second texture is normal texture
    /// If any are absent a default one will be created instead
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: String,
        properties: PropertiesUniform,
        layout: &wgpu::BindGroupLayout,
        diffuse_texture: Option<texture::Texture>,
        normal_texture: Option<texture::Texture>,
    ) -> Self {
        let textures = [
            &diffuse_texture.unwrap_or_else(|| {
                texture::Texture::default_diffuse(
                    device,
                    queue,
                    Some(&format!("Default Diffuse: {name}")),
                )
            }),
            &normal_texture.unwrap_or_else(|| {
                texture::Texture::default_normal(
                    device,
                    queue,
                    Some(&format!("Default Normal: {name}")),
                )
            }),
        ];

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Uniform Buffer: {name}")),
            contents: bytemuck::cast_slice(&[properties]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = texture::util::create_bind_group(
            device,
            layout,
            &textures,
            Some(&buffer),
            Some(&format!("Bind Group: {name}")),
        );

        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&[properties]));

        Self {
            name,
            bind_group,
            properties,
            buffer,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct PropertiesUniform {
    pub diffuse_color: Vec4,
}

impl PropertiesUniform {
    pub const DEFAULT_MAT: Self = PropertiesUniform {
        diffuse_color: Vec4::new(1.0, 0.0, 1.0, 1.0),
    };
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
    fn draw_mesh_instanced(
        &mut self,
        mesh: &Mesh,
        material: &Material,
        instances: Range<u32>,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
        env_bind_group: &wgpu::BindGroup,
    ) {
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        self.set_bind_group(3, env_bind_group, &[]);

        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.draw_indexed(0..mesh.num_indices, 0, instances);
    }
}
