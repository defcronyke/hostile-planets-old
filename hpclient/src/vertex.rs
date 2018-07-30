#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct Vertex {
  pub a_Pos: [f32; 3],
  pub a_Uv: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct Vertex2D {
  pub a_Pos: [f32; 2],
  pub a_Uv: [f32; 2],
}

// gfx_vertex_struct!( Vertex {
//     a_pos: [i8; 4] = "a_pos",
//     a_tex_coord: [i8; 2] = "a_tex_coord",
// });

// impl Vertex {
//     pub fn new(pos: [i8; 3], tc: [i8; 2]) -> Self {
//         Self {
//             a_pos: [pos[0], pos[1], pos[2], 1],
//             a_tex_coord: tc,
//         }
//     }
// }
