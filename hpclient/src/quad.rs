use vertex::Vertex;

pub const QUAD: [Vertex; 6] = [
  Vertex {
    a_Pos: [-0.5, 0.33],
    a_Uv: [0.0, 1.0],
  },
  Vertex {
    a_Pos: [0.5, 0.33],
    a_Uv: [1.0, 1.0],
  },
  Vertex {
    a_Pos: [0.5, -0.33],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [-0.5, 0.33],
    a_Uv: [0.0, 1.0],
  },
  Vertex {
    a_Pos: [0.5, -0.33],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [-0.5, -0.33],
    a_Uv: [0.0, 0.0],
  },
];
