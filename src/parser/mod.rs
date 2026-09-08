use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Result;
use glam::Vec2;
use glam::Vec3;
use log::warn;
use wgpu::BindGroupLayout;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::create_asset_path;
use crate::load_asset_bytes;
use crate::load_asset_string;
use crate::model::Material;
use crate::model::Mesh;
use crate::model::Model;
use crate::model::ModelVertex;
use crate::texture;
use crate::texture::Texture;

pub fn load_model_from_obj(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &BindGroupLayout,
    file_path: impl AsRef<Path>,
) -> anyhow::Result<Model> {
    let obj_path = create_asset_path(&file_path);
    let obj_parent = obj_path.parent().unwrap_or_else(|| Path::new("."));
    let obj_buf = &mut File::open_buffered(&obj_path)?;

    let (objects, object_materials) = {
        let r = tobj::load_obj_buf(obj_buf, &tobj::GPU_LOAD_OPTIONS, |mtl_file| {
            let mtl_buf = &mut File::open_buffered(obj_parent.join(mtl_file))
                .map_err(|_| tobj::LoadError::OpenFileFailed)?;

            tobj::load_mtl_buf(mtl_buf)
        })?;

        (r.0, r.1?)
    };

    let materials = create_model_materials(device, queue, object_materials, layout, |texture| {
        let texture_path = obj_parent.join(texture);
        crate::load_asset_bytes(texture_path)
    })?;

    let mut meshes = Vec::with_capacity(objects.len());

    for object in objects {
        let num_vertices = object.mesh.positions.len() / 3;
        let mut vertices = Vec::with_capacity(num_vertices);

        for i in 0..num_vertices {
            let position = Vec3 {
                x: object.mesh.positions[i * 3],
                y: object.mesh.positions[i * 3 + 1],
                z: object.mesh.positions[i * 3 + 2],
            };

            let texture_uv = if object.mesh.texcoords.is_empty() {
                Vec2::ZERO
            } else {
                Vec2 {
                    x: object.mesh.texcoords[i * 2],
                    y: object.mesh.texcoords[i * 2 + 1],
                }
            };

            let normal = if object.mesh.normals.is_empty() {
                Vec3::ZERO
            } else {
                Vec3 {
                    x: object.mesh.normals[i * 3],
                    y: object.mesh.normals[i * 3 + 1],
                    z: object.mesh.normals[i * 3 + 2],
                }
            };

            vertices.push(ModelVertex {
                position,
                texture_uv,
                normal,
                // We will calculate this later
                tangent: Vec3::ZERO,
                bitanget: Vec3::ZERO,
            });
        }

        // calculate_tangents_and_bitangents(&object.mesh.indices, &mut vertices);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Vertex Buffer: {}", object.name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer: {}", object.name)),
            contents: bytemuck::cast_slice(&object.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        meshes.push(Mesh {
            name: object.name,
            vertex_buffer,
            index_buffer,
            num_indices: object.mesh.indices.len() as u32,
            material_id: object.mesh.material_id,
        });
    }

    Ok(Model { meshes, materials })
}

fn calculate_tangents_and_bitangents(indices: &[u32], vertices: &mut [ModelVertex]) {
    // How many triangles each vertex is used in
    let mut triangles_included = vec![0u32; indices.len()];

    for c in indices.as_chunks::<3>().0 {
        let c0 = c[0] as usize;
        let c1 = c[1] as usize;
        let c2 = c[2] as usize;

        let delta_pos1 = vertices[c1].position - vertices[c0].position;
        let delta_pos2 = vertices[c2].position - vertices[c0].position;

        let delta_uv1 = vertices[c1].texture_uv - vertices[c0].texture_uv;
        let delta_uv2 = vertices[c2].texture_uv - vertices[c0].texture_uv;

        // Solving the following system of equations will
        // give us the tangent and bitangent.
        //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
        //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

        vertices[c0].tangent += tangent;
        vertices[c1].tangent += tangent;
        vertices[c2].tangent += tangent;

        vertices[c0].bitanget += bitangent;
        vertices[c1].bitanget += bitangent;
        vertices[c2].bitanget += bitangent;

        triangles_included[c[0] as usize] += 1;
        triangles_included[c[1] as usize] += 1;
        triangles_included[c[2] as usize] += 1;
    }

    // Average the tangent/bitangent
    for (i, n) in triangles_included.into_iter().enumerate() {
        let denom = 1.0 / n as f32;
        let v = &mut vertices[i];

        v.tangent *= denom;
        v.bitanget *= denom;
    }
}

fn create_model_materials<F>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    object_materials: Vec<tobj::Material>,
    layout: &BindGroupLayout,
    read_texture: F,
) -> anyhow::Result<Vec<Material>>
where
    F: Fn(&str) -> std::io::Result<Vec<u8>>,
{
    let mut materials = Vec::with_capacity(object_materials.len());
    for mat in object_materials {
        let diffuse_texture = if let Some(ref tex) = mat.diffuse_texture {
            Texture::from_bytes(
                device,
                queue,
                &read_texture(tex)?,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                Some(&format!("Diffuse Texture: {}", mat.name)),
            )?
        } else {
            Texture::from_solid_color(
                device,
                queue,
                [1.0, 1.0, 1.0],
                Some(&format!("Diffuse Texture: {}", mat.name)),
            )
        };

        let normal_texture = if let Some(ref tex) = mat.normal_texture {
            Texture::from_bytes(
                device,
                queue,
                &read_texture(tex)?,
                wgpu::TextureFormat::Rgba8Unorm,
                Some(&format!("Normal Texture: {}", mat.name)),
            )?
        } else {
            Texture::create_solid_normal(
                device,
                queue,
                Some(&format!("Normal Texture: {}", mat.name)),
            )
        };

        let bind_group = texture::create_bind_group(
            device,
            layout,
            &[diffuse_texture, normal_texture],
            Some(&format!("Bind Group: {}", mat.name)),
        );

        materials.push(Material {
            name: mat.name,
            bind_group,
        });
    }

    Ok(materials)
}
