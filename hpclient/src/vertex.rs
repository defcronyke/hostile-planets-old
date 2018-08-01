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
