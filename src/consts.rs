#![allow(unused)]

use crate::vertex::TexturedVertex;
use crate::vertex::Vertex;

#[rustfmt::skip]
pub const TEXTURED_CUBE_VERTICES: &[TexturedVertex] = &[
    // Front face (+Z)
    TexturedVertex { position: [-0.5, -0.5,  0.5], texture_coordinates: [0.0, 1.0] },
    TexturedVertex { position: [ 0.5, -0.5,  0.5], texture_coordinates: [1.0, 1.0] },
    TexturedVertex { position: [ 0.5,  0.5,  0.5], texture_coordinates: [1.0, 0.0] },
    TexturedVertex { position: [-0.5,  0.5,  0.5], texture_coordinates: [0.0, 0.0] },

    // Back face (-Z)
    TexturedVertex { position: [ 0.5, -0.5, -0.5], texture_coordinates: [0.0, 1.0] },
    TexturedVertex { position: [-0.5, -0.5, -0.5], texture_coordinates: [1.0, 1.0] },
    TexturedVertex { position: [-0.5,  0.5, -0.5], texture_coordinates: [1.0, 0.0] },
    TexturedVertex { position: [ 0.5,  0.5, -0.5], texture_coordinates: [0.0, 0.0] },

    // Top face (+Y)
    TexturedVertex { position: [-0.5,  0.5,  0.5], texture_coordinates: [0.0, 1.0] },
    TexturedVertex { position: [ 0.5,  0.5,  0.5], texture_coordinates: [1.0, 1.0] },
    TexturedVertex { position: [ 0.5,  0.5, -0.5], texture_coordinates: [1.0, 0.0] },
    TexturedVertex { position: [-0.5,  0.5, -0.5], texture_coordinates: [0.0, 0.0] },

    // Bottom face (-Y)
    TexturedVertex { position: [-0.5, -0.5, -0.5], texture_coordinates: [0.0, 1.0] },
    TexturedVertex { position: [ 0.5, -0.5, -0.5], texture_coordinates: [1.0, 1.0] },
    TexturedVertex { position: [ 0.5, -0.5,  0.5], texture_coordinates: [1.0, 0.0] },
    TexturedVertex { position: [-0.5, -0.5,  0.5], texture_coordinates: [0.0, 0.0] },

    // Right face (+X)
    TexturedVertex { position: [ 0.5, -0.5,  0.5], texture_coordinates: [0.0, 1.0] },
    TexturedVertex { position: [ 0.5, -0.5, -0.5], texture_coordinates: [1.0, 1.0] },
    TexturedVertex { position: [ 0.5,  0.5, -0.5], texture_coordinates: [1.0, 0.0] },
    TexturedVertex { position: [ 0.5,  0.5,  0.5], texture_coordinates: [0.0, 0.0] },

    // Left face (-X)
    TexturedVertex { position: [-0.5, -0.5, -0.5], texture_coordinates: [0.0, 1.0] },
    TexturedVertex { position: [-0.5, -0.5,  0.5], texture_coordinates: [1.0, 1.0] },
    TexturedVertex { position: [-0.5,  0.5,  0.5], texture_coordinates: [1.0, 0.0] },
    TexturedVertex { position: [-0.5,  0.5, -0.5], texture_coordinates: [0.0, 0.0] },
];

pub const TEXTURED_CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // front
    4, 5, 6, 4, 6, 7, // back
    8, 9, 10, 8, 10, 11, // top
    12, 13, 14, 12, 14, 15, // bottom
    16, 17, 18, 16, 18, 19, // right
    20, 21, 22, 20, 22, 23, // left
];

pub const NUM_INSTANCES_PER_ROW: u32 = 5;

pub const INSTANCE_DISPLACEMENT: glam::Vec3 = glam::Vec3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);
