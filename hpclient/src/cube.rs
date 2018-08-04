use cgmath::{Matrix4, SquareMatrix};
use object::Object;
use vertex::Vertex;

pub struct Cube {
  pub vertices: [Vertex; 24],
  pub indices: [u16; 36],
  pub model_matrix: Matrix4<f32>,
}

impl Cube {
  pub fn new() -> Self {
    Self {
      vertices: CUBE_VERTICES,
      indices: CUBE_INDICES,
      model_matrix: Matrix4::identity(),
    }
  }
}

impl Object for Cube {}

pub const CUBE_VERTICES: [Vertex; 24] = [
  // front
  Vertex {
    a_pos: [-1.0, -1.0, 1.0],
    a_uv: [0.0, 1.0],
  },
  Vertex {
    a_pos: [1.0, -1.0, 1.0],
    a_uv: [1.0, 1.0],
  },
  Vertex {
    a_pos: [1.0, 1.0, 1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, 1.0],
    a_uv: [0.0, 0.0],
  },
  // back
  Vertex {
    a_pos: [1.0, 1.0, -1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, -1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, -1.0, -1.0],
    a_uv: [1.0, 1.0],
  },
  Vertex {
    a_pos: [1.0, -1.0, -1.0],
    a_uv: [0.0, 1.0],
  },
  // right
  Vertex {
    a_pos: [1.0, -1.0, -1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [1.0, 1.0, -1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [1.0, 1.0, 1.0],
    a_uv: [0.0, 1.0],
  },
  Vertex {
    a_pos: [1.0, -1.0, 1.0],
    a_uv: [1.0, 1.0],
  },
  // left
  Vertex {
    a_pos: [-1.0, 1.0, 1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, -1.0, 1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, -1.0, -1.0],
    a_uv: [1.0, 1.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, -1.0],
    a_uv: [0.0, 1.0],
  },
  // top
  Vertex {
    a_pos: [-1.0, 1.0, -1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [1.0, 1.0, -1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [1.0, 1.0, 1.0],
    a_uv: [1.0, 1.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, 1.0],
    a_uv: [0.0, 1.0],
  },
  // bottom
  Vertex {
    a_pos: [1.0, -1.0, 1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, -1.0, 1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, -1.0, -1.0],
    a_uv: [0.0, 1.0],
  },
  Vertex {
    a_pos: [1.0, -1.0, -1.0],
    a_uv: [1.0, 1.0],
  },
];

pub const CUBE_INDICES: [u16; 36] = [
  0, 1, 2, 2, 3, 0, // front
  4, 6, 5, 6, 4, 7, // back
  8, 9, 10, 10, 11, 8, // right
  12, 14, 13, 14, 12, 15, // left
  16, 18, 17, 18, 16, 19, // top
  20, 21, 22, 22, 23, 20, // bottom
];
