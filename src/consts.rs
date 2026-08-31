#![allow(unused)]

use crate::primitives::TexturedVertex;
use crate::primitives::Vertex;

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

pub const TEXTURED_VERTICES: &[TexturedVertex] = &[
    // Changed
    TexturedVertex {
        position: [-0.0868241, 0.49240386, 0.0],
        texture_coordinates: [0.4131759, 0.00759614],
    }, // A
    TexturedVertex {
        position: [-0.49513406, 0.06958647, 0.0],
        texture_coordinates: [0.0048659444, 0.43041354],
    }, // B
    TexturedVertex {
        position: [-0.21918549, -0.44939706, 0.0],
        texture_coordinates: [0.28081453, 0.949397],
    }, // C
    TexturedVertex {
        position: [0.35966998, -0.3473291, 0.0],
        texture_coordinates: [0.85967, 0.84732914],
    }, // D
    TexturedVertex {
        position: [0.44147372, 0.2347359, 0.0],
        texture_coordinates: [0.9414737, 0.2652641],
    }, // E
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
