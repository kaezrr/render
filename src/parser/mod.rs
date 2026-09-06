use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use log::warn;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupLayout;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;

use crate::load_asset_bytes;
use crate::load_asset_string;
use crate::model::Material;
use crate::model::Mesh;
use crate::model::Model;
use crate::model::ModelVertex;
use crate::texture::Texture;

pub fn load_model_from_obj(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &BindGroupLayout,
    file_path: impl AsRef<Path>,
) -> anyhow::Result<Model> {
    let file_str = load_asset_string(&file_path)?;
    let parent_path = file_path
        .as_ref()
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let parsed_obj = parse_obj_file(&file_str, |mtl_file| {
        let mtl_path = parent_path.join(mtl_file);
        load_asset_string(mtl_path)
    })?;

    let meshes = construct_meshes(device, &parsed_obj)?;
    let materials = construct_materials(
        device,
        queue,
        layout,
        parsed_obj.materials,
        |texture_file| {
            let texture_path = parent_path.join(texture_file);
            load_asset_bytes(texture_path)
        },
    )?;

    Ok(Model { meshes, materials })
}

fn construct_materials<F>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &BindGroupLayout,
    object_materials: Vec<ObjectMaterial>,
    read_texture: F,
) -> anyhow::Result<Vec<Material>>
where
    F: Fn(&str) -> std::io::Result<Vec<u8>>,
{
    let mut materials: Vec<Material> = Vec::new();

    for material in object_materials {
        // TODO: Dont skip materials without a diffuse texture
        let Some(diffuse_texture_map) = material.diffuse_map else {
            continue;
        };

        let dtexture_bytes = read_texture(&diffuse_texture_map)?;
        let diffuse_texture = Texture::from_bytes(device, queue, &dtexture_bytes, &material.name)?;

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("Bind Group: {}", material.name)),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        materials.push(Material {
            name: material.name,
            _diffuse_texture: diffuse_texture,
            bind_group,
        });
    }

    Ok(materials)
}

fn construct_meshes(device: &wgpu::Device, obj: &ParsedObj) -> anyhow::Result<Vec<Mesh>> {
    let mut meshes: Vec<Mesh> = Vec::new();

    for object in &obj.objects {
        // Group faces by material ids
        let mut groups: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
        for (i, mat_id) in object.faces_material_id.iter().enumerate() {
            groups.entry(*mat_id).or_default().push(i);
        }

        for (material_id, face_indices) in groups {
            let mut vertices: Vec<ModelVertex> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            let mut vertex_hashmap: HashMap<(i32, Option<i32>, Option<i32>), u32> = HashMap::new();

            for face_i in face_indices {
                let face = &object.faces[face_i];
                let resolved: Vec<u32> = face
                    .iter()
                    .map(|index| -> anyhow::Result<_> {
                        let key = (index.vertex, index.texture, index.normal);
                        if let Some(&existing) = vertex_hashmap.get(&key) {
                            Ok(existing)
                        } else {
                            vertices.push(construct_vertex_from_index(obj, index)?);
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
                label: Some(&format!("Vertex Buffer: {}", object.name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("Index Buffer: {}", object.name)),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            meshes.push(Mesh {
                name: object.name.clone(),
                vertex_buffer,
                index_buffer,
                num_indices: indices.len() as u32,
                material_id,
            });
        }
    }

    Ok(meshes)
}

fn parse_obj_file<F>(file_str: &str, read_mtl: F) -> anyhow::Result<ParsedObj>
where
    F: Fn(&str) -> std::io::Result<String>,
{
    let mut parsed_obj = ParsedObj::default();
    let mut current_material: Option<&str> = None;

    for line in file_str.lines() {
        let tokens = line.split_whitespace().collect::<Vec<_>>();

        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            // Geometry Vertex
            "v" => {
                parsed_obj.geometry_vertices.push([
                    tokens[1].parse()?,
                    tokens[2].parse()?,
                    tokens[3].parse()?,
                ]);
            }

            // Texture coordinate
            "vt" => {
                parsed_obj.texture_uvs.push([
                    tokens[1].parse()?,
                    tokens.get(2).map(|x| x.parse()).transpose()?.unwrap_or(0.0),
                ]);
            }

            // Vertex Normal
            "vn" => {
                parsed_obj.vertex_normals.push([
                    tokens[1].parse()?,
                    tokens[2].parse()?,
                    tokens[3].parse()?,
                ]);
            }

            // Polygon Face
            "f" => {
                let face_indices = tokens
                    .iter()
                    .skip(1)
                    .map(|x| -> anyhow::Result<_> {
                        let indices = x.split('/').collect::<Vec<_>>();

                        Ok(Index {
                            vertex: indices[0].parse()?,
                            texture: indices.get(1).map(|&x| x.parse()).transpose()?,
                            normal: indices.get(2).map(|&x| x.parse()).transpose()?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let object = parsed_obj
                    .objects
                    .last_mut()
                    .ok_or(anyhow::anyhow!("face without parent object"))?;

                object.faces.push(face_indices);

                object.faces_material_id.push(
                    current_material.and_then(|curr| {
                        parsed_obj.materials.iter().position(|mat| mat.name == curr)
                    }),
                );
            }

            // Model begin
            "o" => parsed_obj.objects.push(Object {
                name: tokens[1].to_owned(),
                ..Default::default()
            }),

            // Material file
            "mtllib" => {
                let mtl_file_str = read_mtl(tokens[1])?;
                let material = parse_mtl_file(&mtl_file_str)?;
                parsed_obj.materials.extend(material);
            }

            // Set Face Material
            "usemtl" => current_material = Some(tokens[1]),

            // Smoothing Group
            "s" => warn!("obj_parser: ignoring smoothing group 's'"),

            // Line Element
            "l" => warn!("obj_parser: ignoring polyline 'l'"),

            // Comment
            "#" => {}

            x => anyhow::bail!("obj_parser: unknown obj line beginning with '{x}'"),
        }
    }

    Ok(parsed_obj)
}

fn parse_mtl_file(file_str: &str) -> anyhow::Result<Vec<ObjectMaterial>> {
    let mut parsed_materials: Vec<ObjectMaterial> = Vec::new();

    for line in file_str.lines() {
        let tokens = line.split_whitespace().collect::<Vec<_>>();

        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            // New Material
            "newmtl" => parsed_materials.push(ObjectMaterial {
                name: tokens[1].to_owned(),
                ..Default::default()
            }),

            "map_Bump" => {
                parsed_materials
                    .last_mut()
                    .ok_or(anyhow::anyhow!("normal map without parent material"))?
                    .normal_map = Some(tokens[1].to_owned());
            }

            "map_Kd" => {
                parsed_materials
                    .last_mut()
                    .ok_or(anyhow::anyhow!("diffuse map without parent material"))?
                    .diffuse_map = Some(tokens[1].to_owned());
            }

            #[rustfmt::skip]
            // Diffuse color
            "Kd" => {
                parsed_materials
                    .last_mut()
                    .ok_or(anyhow::anyhow!("diffuse color without parent material"))?
                    .diffuse_color = Some([
                        tokens[1].parse()?,
                        tokens[2].parse()?,
                        tokens[3].parse()?
                    ]);
            }

            // Comment
            "#" => {}

            "Ks" => warn!("obj_parser: ignoring specular color 'Ks'"),

            "Ns" => warn!("obj_parser: ignoring specular color exponent 'Ns'"),

            "Ka" => warn!("obj_parser: ignoring ambient color 'Ka'"),

            "Ke" => warn!("obj_parser: ignoring emissive color 'Ke'"),

            "Ni" => warn!("obj_parser: ignoring optical density 'Ni'"),

            "d" => warn!("obj_parser: ignoring dissolve 'd'"),

            "illum" => warn!("obj_parser: ignoring illumination model 'illum'"),

            x => anyhow::bail!("obj_parser: unknown mtl line beginning with '{x}'"),
        }
    }

    Ok(parsed_materials)
}

#[derive(Debug, Default)]
struct ParsedObj {
    geometry_vertices: Vec<[f32; 3]>,
    texture_uvs: Vec<[f32; 2]>,
    vertex_normals: Vec<[f32; 3]>,

    objects: Vec<Object>,
    materials: Vec<ObjectMaterial>,
}

#[derive(Debug, Default)]
struct Object {
    name: String,
    faces: Vec<Vec<Index>>,
    faces_material_id: Vec<Option<usize>>,
}

#[derive(Debug)]
struct Index {
    vertex: i32,
    texture: Option<i32>,
    normal: Option<i32>,
}

#[derive(Debug, Default)]
struct ObjectMaterial {
    name: String,
    normal_map: Option<String>,
    diffuse_map: Option<String>,
    diffuse_color: Option<[f32; 3]>,
}

/// Get at an offset starting from 1
fn get_at_offset<T: Clone + Copy>(buffer: &[T], offset: i32) -> Option<T> {
    if offset.is_negative() {
        let n = offset.unsigned_abs() as usize;
        return buffer.iter().nth_back(n - 1).copied();
    }

    let n = offset.unsigned_abs() as usize;
    buffer.get(n - 1).copied()
}

fn construct_vertex_from_index(obj: &ParsedObj, index: &Index) -> anyhow::Result<ModelVertex> {
    let position = get_at_offset(&obj.geometry_vertices, index.vertex)
        .ok_or(anyhow::anyhow!("invalid vertex data"))?;

    let texture_uv = index
        .texture
        .and_then(|i| get_at_offset(&obj.texture_uvs, i))
        .unwrap_or_default();

    let normal = index
        .normal
        .and_then(|i| get_at_offset(&obj.vertex_normals, i))
        .unwrap_or_default();

    Ok(ModelVertex {
        position,
        texture_uv,
        normal,
    })
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::load_asset_string;
    use crate::parser::parse_obj_file;

    #[test]
    fn parse_obj() {
        let file_path = Path::new("models/cube/cube.obj");
        let file_str = load_asset_string(file_path).expect("obj file should load");

        let result = parse_obj_file(&file_str, |mtl_file| {
            let mtl_path = file_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(mtl_file);

            load_asset_string(mtl_path)
        });

        result.expect("failed to parse obj");
    }
}
