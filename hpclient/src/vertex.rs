#[derive(Debug, Clone, Copy)]
pub struct Vertex {
  pub a_pos: [f32; 3],
  pub a_uv: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex2D {
  pub a_pos: [f32; 2],
  pub a_uv: [f32; 2],
}
