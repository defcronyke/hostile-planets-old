use object::Object;
use vertex::Vertex2D;

pub struct Quad {
  pub vertices: [Vertex2D; 6],
}

impl Quad {
  pub fn new() -> Self {
    Self {
      vertices: QUAD_VERTICES,
    }
  }
}

impl Object for Quad {
  fn get_name(&self) -> String {
    String::from("a quad")
  }
}

pub const QUAD_VERTICES: [Vertex2D; 6] = [
  Vertex2D {
    a_pos: [-0.5, 0.33],
    a_uv: [0.0, 1.0],
  },
  Vertex2D {
    a_pos: [0.5, 0.33],
    a_uv: [1.0, 1.0],
  },
  Vertex2D {
    a_pos: [0.5, -0.33],
    a_uv: [1.0, 0.0],
  },
  Vertex2D {
    a_pos: [-0.5, 0.33],
    a_uv: [0.0, 1.0],
  },
  Vertex2D {
    a_pos: [0.5, -0.33],
    a_uv: [1.0, 0.0],
  },
  Vertex2D {
    a_pos: [-0.5, -0.33],
    a_uv: [0.0, 0.0],
  },
];
