use object::*;
use window::*;

use gltf;
use std::io;
use vecmath::*;
use piston_window::*;
use piston_window::Window;
use camera_controllers::{
    CameraPerspective,
    model_view_projection,
    FirstPerson
};

#[derive(Debug, Clone)]
pub struct GltfObject {
  pub data: gltf::Gltf,
  pub model: Matrix4<f32>,
  pub projection: Matrix4<f32>,
  // pub u_model_view_proj: Matrix4<f32>,
}

impl GltfObject {
  // pub fn init(&mut self, pw: &mut _PistonWindow) -> io::Result<i32> {
  //   let w = &pw.window;
  //   let ogl = pw.opengl;
  //   let ref mut f = w.factory.clone();
  //   let (vbuf, slice) = f.create_vertex_buffer_with_slice(&cube_vertex_data, cube_index_data.as_slice());
  //   let (_, texture_view) = f.create_texture_immutable::<gfx::format::Rgba8>(
  //     gfx::texture::Kind::D2(2, 2, gfx::texture::AaMode::Single),
  //       gfx::texture::Mipmap::Provided,
  //       &[cube_texel_data.as_slice()]).unwrap();

  //   let sinfo = gfx::texture::SamplerInfo::new(
  //   gfx::texture::FilterMethod::Bilinear,
  //   gfx::texture::WrapMode::Clamp);

  //   let glsl = ogl.to_glsl();
  //   let pso = f.create_pipeline_simple(
  //     Shaders::new()
  //       .set(GLSL::V1_20, include_str!("../assets/shaders/cube_120.glslv"))
  //       .set(GLSL::V1_50, include_str!("../assets/shaders/cube_150.glslv"))
  //       .get(glsl).unwrap().as_bytes(),
  //     Shaders::new()
  //       .set(GLSL::V1_20, include_str!("../assets/shaders/cube_120.glslf"))
  //       .set(GLSL::V1_50, include_str!("../assets/shaders/cube_150.glslf"))
  //       .get(glsl).unwrap().as_bytes(),
  //     pipe::new()
  //   ).unwrap();

  //   let model = vecmath::mat4_id();
  //   let projection = Self::get_projection(&w);
  //   // let first_person = FirstPerson::new(
  //   //     [0.5, 0.5, 4.0],
  //   //     FirstPersonSettings::keyboard_wasd()
  //   // );

  //   let data = pipe::Data {
  //     vbuf: vbuf.clone(),
  //     u_model_view_proj: [[0.0; 4]; 4],
  //     t_color: (texture_view, f.create_sampler(sinfo)),
  //     out_color: w.output_color.clone(),
  //     out_depth: w.output_stencil.clone(),
  //   };

  //   Ok(0)
  // }

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

  pub fn set_projection(&mut self, w: &mut PistonWindow) -> io::Result<i32> {
    self.projection = GltfObject::get_projection(&w);

    Ok(0)
  }
}

impl Object for GltfObject {
  fn get_name(&self) -> String {
    String::from("a gltf object")
  }

  fn draw(&mut self, w: &mut PistonWindow, args: &RenderArgs, first_person: &FirstPerson) -> io::Result<i32> {
    // self.u_model_view_proj = model_view_projection(
    //   self.model,
    //   first_person.camera(args.ext_dt).orthogonal(),
    //   self.projection
    // );
    // w.encoder.draw(&self.slice, &self.pso, &self.data);

    Ok(0)
  }
}
