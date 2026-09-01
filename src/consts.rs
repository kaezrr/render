#![allow(unused)]

use crate::vertex::TexturedVertex;
use crate::vertex::Vertex;

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

pub const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    }, // top, red
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    }, // bottom left, green
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    }, // bottom right, blue
];

pub const TRIANGLE_INDICES: &[u16] = &[0, 1, 2];

pub const CUBE_VERTICES: &[Vertex] = &[
    // Front face (+Z), red
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    // Back face (-Z), green
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    // Top face (+Y), blue
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    // Bottom face (-Y), yellow
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    // Right face (+X), magenta
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    // Left face (-X), cyan
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    },
];

pub const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // front
    4, 5, 6, 4, 6, 7, // back
    8, 9, 10, 8, 10, 11, // top
    12, 13, 14, 12, 14, 15, // bottom
    16, 17, 18, 16, 18, 19, // right
    20, 21, 22, 20, 22, 23, // left
];
