// use piston_window::*;

// use std::io;
// use camera_controllers::FirstPerson;
// use vecmath::*;

pub trait Object: Send + Sync {
  fn get_name(&self) -> String {
    String::from("an unknown object")
  }

  // fn draw(&mut self, _w: &mut PistonWindow, _args: &RenderArgs, _first_person: &FirstPerson) -> io::Result<i32> {
  //   Err(io::Error::from(io::ErrorKind::NotFound))
  // }

  // fn reset(&mut self, _w: &mut PistonWindow) -> io::Result<i32> {
  //   Err(io::Error::from(io::ErrorKind::NotFound))
  // }

  // fn get_projection(_w: &PistonWindow) -> Matrix4<f32> {
  //   mat4_id()
  // }
}
