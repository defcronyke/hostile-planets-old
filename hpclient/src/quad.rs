use vertex::Vertex2D;

pub const QUAD: [Vertex2D; 6] = [
  Vertex2D {
    a_Pos: [-0.5, 0.33],
    a_Uv: [0.0, 1.0],
  },
  Vertex2D {
    a_Pos: [0.5, 0.33],
    a_Uv: [1.0, 1.0],
  },
  Vertex2D {
    a_Pos: [0.5, -0.33],
    a_Uv: [1.0, 0.0],
  },
  Vertex2D {
    a_Pos: [-0.5, 0.33],
    a_Uv: [0.0, 1.0],
  },
  Vertex2D {
    a_Pos: [0.5, -0.33],
    a_Uv: [1.0, 0.0],
  },
  Vertex2D {
    a_Pos: [-0.5, -0.33],
    a_Uv: [0.0, 0.0],
  },
];
