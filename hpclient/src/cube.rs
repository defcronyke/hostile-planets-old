use cgmath::{Matrix4, SquareMatrix};
use object::Object;
use vertex::Vertex;

pub struct Cube {
  pub vertices: [Vertex; 24],
  pub indices: [u16; 36],
  pub texels: [[u8; 4]; 4],
  pub model_matrix: Matrix4<f32>,
}

impl Cube {
  pub fn new() -> Self {
    Self {
      vertices: CUBE_VERTICES,
      indices: CUBE_INDICES,
      texels: CUBE_TEXELS,
      model_matrix: Matrix4::identity(),
    }
  }
}

impl Object for Cube {}

pub const CUBE_VERTICES: [Vertex; 24] = [
  //top (0, 0, 1)
  Vertex {
    a_pos: [-1.0, -1.0, 1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [1.0, -1.0, 1.0],
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
  //bottom (0, 0, -1)
  Vertex {
    a_pos: [-1.0, 1.0, -1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [1.0, 1.0, -1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [1.0, -1.0, -1.0],
    a_uv: [0.0, 1.0],
  },
  Vertex {
    a_pos: [-1.0, -1.0, -1.0],
    a_uv: [1.0, 1.0],
  },
  //right (1, 0, 0)
  Vertex {
    a_pos: [1.0, -1.0, -1.0],
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
    a_pos: [1.0, -1.0, 1.0],
    a_uv: [0.0, 1.0],
  },
  //left (-1, 0, 0)
  Vertex {
    a_pos: [-1.0, -1.0, 1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, 1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, -1.0],
    a_uv: [0.0, 1.0],
  },
  Vertex {
    a_pos: [-1.0, -1.0, -1.0],
    a_uv: [1.0, 1.0],
  },
  //front (0, 1, 0)
  Vertex {
    a_pos: [1.0, 1.0, -1.0],
    a_uv: [1.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, -1.0],
    a_uv: [0.0, 0.0],
  },
  Vertex {
    a_pos: [-1.0, 1.0, 1.0],
    a_uv: [0.0, 1.0],
  },
  Vertex {
    a_pos: [1.0, 1.0, 1.0],
    a_uv: [1.0, 1.0],
  },
  //back (0, -1, 0)
  Vertex {
    a_pos: [1.0, -1.0, 1.0],
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
    a_pos: [1.0, -1.0, -1.0],
    a_uv: [0.0, 1.0],
  },
];

// pub const CUBE_VERTICES: [Vertex; 24] = [
//   //top (0, 0, 1)
//   Vertex {
//     a_pos: [-1.0, -1.0, 1.0],
//     a_uv: [0.0, 0.0],
//   },
//   Vertex {
//     a_pos: [1.0, -1.0, 1.0],
//     a_uv: [1.0, 0.0],
//   },
//   Vertex {
//     a_pos: [1.0, 1.0, 1.0],
//     a_uv: [1.0, 1.0],
//   },
//   Vertex {
//     a_pos: [-1.0, 1.0, 1.0],
//     a_uv: [0.0, 1.0],
//   },
//   //bottom (0, 0, -1)
//   Vertex {
//     a_pos: [1.0, 1.0, -1.0],
//     a_uv: [0.0, 0.0],
//   },
//   Vertex {
//     a_pos: [-1.0, 1.0, -1.0],
//     a_uv: [1.0, 0.0],
//   },
//   Vertex {
//     a_pos: [-1.0, -1.0, -1.0],
//     a_uv: [1.0, 1.0],
//   },
//   Vertex {
//     a_pos: [1.0, -1.0, -1.0],
//     a_uv: [0.0, 1.0],
//   },
//   //right (1, 0, 0)
//   Vertex {
//     a_pos: [1.0, -1.0, -1.0],
//     a_uv: [0.0, 0.0],
//   },
//   Vertex {
//     a_pos: [1.0, 1.0, -1.0],
//     a_uv: [1.0, 0.0],
//   },
//   Vertex {
//     a_pos: [1.0, 1.0, 1.0],
//     a_uv: [1.0, 1.0],
//   },
//   Vertex {
//     a_pos: [1.0, -1.0, 1.0],
//     a_uv: [0.0, 1.0],
//   },
//   //left (-1, 0, 0)
//   Vertex {
//     a_pos: [-1.0, 1.0, 1.0],
//     a_uv: [0.0, 0.0],
//   },
//   Vertex {
//     a_pos: [-1.0, -1.0, 1.0],
//     a_uv: [1.0, 0.0],
//   },
//   Vertex {
//     a_pos: [-1.0, -1.0, -1.0],
//     a_uv: [1.0, 1.0],
//   },
//   Vertex {
//     a_pos: [-1.0, 1.0, -1.0],
//     a_uv: [0.0, 1.0],
//   },
//   //front (0, 1, 0)
//   Vertex {
//     a_pos: [-1.0, 1.0, -1.0],
//     a_uv: [0.0, 0.0],
//   },
//   Vertex {
//     a_pos: [1.0, 1.0, -1.0],
//     a_uv: [1.0, 0.0],
//   },
//   Vertex {
//     a_pos: [1.0, 1.0, 1.0],
//     a_uv: [1.0, 1.0],
//   },
//   Vertex {
//     a_pos: [-1.0, 1.0, 1.0],
//     a_uv: [0.0, 1.0],
//   },
//   //back (0, -1, 0)
//   Vertex {
//     a_pos: [1.0, -1.0, 1.0],
//     a_uv: [0.0, 0.0],
//   },
//   Vertex {
//     a_pos: [-1.0, -1.0, 1.0],
//     a_uv: [1.0, 0.0],
//   },
//   Vertex {
//     a_pos: [-1.0, -1.0, -1.0],
//     a_uv: [1.0, 1.0],
//   },
//   Vertex {
//     a_pos: [1.0, -1.0, -1.0],
//     a_uv: [0.0, 1.0],
//   },
// ];

pub const CUBE_INDICES: [u16; 36] = [
  0, 1, 2, 2, 3, 0, // top
  4, 5, 6, 6, 7, 4, // bottom
  8, 9, 10, 10, 11, 8, // right
  12, 13, 14, 14, 15, 12, // left
  16, 17, 18, 18, 19, 16, // front
  20, 21, 22, 22, 23, 20, // back
];

// pub const CUBE_INDICES: [u16; 36] = [
//   0, 1, 2, 2, 3, 0, // top
//   4, 6, 5, 6, 4, 7, // bottom
//   8, 9, 10, 10, 11, 8, // right
//   12, 14, 13, 14, 12, 15, // left
//   16, 18, 17, 18, 16, 19, // front
//   20, 21, 22, 22, 23, 20, // back
// ];

pub const CUBE_TEXELS: [[u8; 4]; 4] = [
  [0xff, 0xff, 0xff, 0x00],
  [0xff, 0x00, 0x00, 0x00],
  [0x00, 0xff, 0x00, 0x00],
  [0x00, 0x00, 0xff, 0x00],
];
