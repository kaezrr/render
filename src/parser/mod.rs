use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use log::warn;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::load_asset_string;
use crate::model::Mesh;
use crate::model::Model;
use crate::model::ModelVertex;

pub fn load_model_from_obj(
    device: &wgpu::Device,
    file_path: impl AsRef<Path>,
) -> anyhow::Result<Model> {
    let file_str = load_asset_string(file_path)?;
    let (objects, object_materials) = parse_obj_file(&file_str)?;

    let mut meshes: Vec<Mesh> = Vec::new();

    for object in objects {
        let mut vertices: Vec<ModelVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_hashmap: HashMap<(i32, Option<i32>, Option<i32>), u32> = HashMap::new();

        for face in &object.faces {
            let resolved: Vec<u32> = face
                .iter()
                .map(|index| -> anyhow::Result<_> {
                    let key = (index.vertex, index.texture, index.normal);
                    if let Some(&existing) = vertex_hashmap.get(&key) {
                        Ok(existing)
                    } else {
                        vertices.push(construct_vertex_from_index(&object, index)?);
                        let value = vertices.len() as u32 - 1;
                        vertex_hashmap.insert(key, value);
                        Ok(value)
                    }
                })
                .collect::<anyhow::Result<_>>()?;

            for i in 1..resolved.len() - 1 {
                indices.push(resolved[0]);
                indices.push(resolved[i]);
                indices.push(resolved[i + 1]);
            }
        }

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        meshes.push(Mesh {
            name: object.name,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            material: todo!(),
        });
    }

    Ok(Model {
        meshes,
        materials: todo!(),
    })
}

fn parse_obj_file(file_str: &str) -> anyhow::Result<(Vec<Object>, Vec<ObjectMaterial>)> {
    let mut parsed_objects: Vec<Object> = Vec::new();
    let mut parsed_materials: Vec<ObjectMaterial> = Vec::new();

    for line in file_str.lines() {
        let tokens = line.split_whitespace().collect::<Vec<_>>();

        match tokens[0] {
            // Geometry Vertex
            "v" => {
                let geometry_vertex = [tokens[1].parse()?, tokens[2].parse()?, tokens[3].parse()?];

                parsed_objects
                    .last_mut()
                    .ok_or(anyhow::anyhow!("geometry vertex without parent object"))?
                    .geometry_vertices
                    .push(geometry_vertex);
            }

            // Texture coordinate
            "vt" => {
                let texture_uv = [
                    tokens[1].parse()?,
                    tokens.get(2).map(|x| x.parse()).transpose()?.unwrap_or(0.0),
                ];

                parsed_objects
                    .last_mut()
                    .ok_or(anyhow::anyhow!("texture coordinate without parent object"))?
                    .texture_uvs
                    .push(texture_uv);
            }

            // Vertex Normal
            "vn" => {
                let vertex_normal = [tokens[1].parse()?, tokens[2].parse()?, tokens[3].parse()?];

                parsed_objects
                    .last_mut()
                    .ok_or(anyhow::anyhow!("vertex normal without parent object"))?
                    .vertex_normals
                    .push(vertex_normal);
            }

            "f" => {
                let face_indices = tokens
                    .iter()
                    .skip(1)
                    .map(|x| -> anyhow::Result<_> {
                        let indices = x.split('/').collect::<Vec<_>>();

                        Ok(Index {
                            vertex: indices[0].parse()?,
                            texture: indices.get(1).map(|x| x.parse()).transpose()?,
                            normal: indices.get(2).map(|x| x.parse()).transpose()?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                parsed_objects
                    .last_mut()
                    .ok_or(anyhow::anyhow!("face without parent object"))?
                    .faces
                    .push(face_indices);
            }

            // Material file
            "mtllib" => warn!("materials not implemented yet: mtllib"),

            "usemtl" => warn!("materials not implemented yet: usemtl"),

            // Model begin
            "o" => {
                parsed_objects.push(Object {
                    name: tokens[1].to_owned(),
                    geometry_vertices: vec![],
                    texture_uvs: vec![],
                    vertex_normals: vec![],
                    faces: vec![],
                });
            }

            // Smoothing Group
            "s" => warn!("ignoring smoothing group"),

            // Comment
            "#" => {}

            x => anyhow::bail!("unknown obj line beginning with \"{x}\""),
        }
    }

    Ok((parsed_objects, parsed_materials))
}

#[derive(Debug)]
struct Object {
    name: String,
    geometry_vertices: Vec<[f32; 3]>,
    texture_uvs: Vec<[f32; 2]>,
    vertex_normals: Vec<[f32; 3]>,
    faces: Vec<Vec<Index>>,
}

#[derive(Debug)]
struct Index {
    vertex: i32,
    texture: Option<i32>,
    normal: Option<i32>,
}

#[derive(Debug)]
struct ObjectMaterial {}

/// Get at an offset starting from 1
fn get_at_offset<T: Clone + Copy>(buffer: &[T], offset: i32) -> Option<T> {
    if offset.is_negative() {
        let n = offset.unsigned_abs() as usize;
        return buffer.iter().nth_back(n - 1).copied();
    }

    let n = offset.unsigned_abs() as usize;
    buffer.get(n - 1).copied()
}

fn construct_vertex_from_index(object: &Object, index: &Index) -> anyhow::Result<ModelVertex> {
    let position = get_at_offset(&object.geometry_vertices, index.vertex)
        .ok_or(anyhow::anyhow!("invalid vertex data"))?;

    let texture_uv = index
        .texture
        .and_then(|i| get_at_offset(&object.texture_uvs, i))
        .unwrap_or_default();

    let normal = index
        .normal
        .and_then(|i| get_at_offset(&object.vertex_normals, i))
        .unwrap_or_default();

    Ok(ModelVertex {
        position,
        texture_uv,
        normal,
    })
}
