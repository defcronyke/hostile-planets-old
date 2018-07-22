use object::*;

use gltf;
use std::io;
use vecmath::*;
use piston_window::*;
use camera_controllers::{
    CameraPerspective
};

#[derive(Debug, Clone)]
pub struct GltfObject {
  pub data: gltf::Gltf,
  pub model: Matrix4<f32>,
  pub projection: Matrix4<f32>,
}

impl GltfObject {
  pub fn reset(&mut self, w: &mut PistonWindow) -> io::Result<i32> {
    self.projection = Self::get_projection(&w);
    // self.data.out_color = w.output_color.clone();
    // self.data.out_depth = w.output_stencil.clone();

    Ok(0)
  }

  pub fn get_projection(w: &PistonWindow) -> Matrix4<f32> {
    let draw_size = w.window.draw_size();
    CameraPerspective {
      fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
      aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
    }.projection()
  }
}

impl Object for GltfObject {
  fn get_name(&self) -> String {
    String::from("a gltf object")
  }
}
