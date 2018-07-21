use piston_window::*;

use std::io;

pub trait Object {
  fn get_name(&self) -> String {
    String::from("unknown")
  }

  fn draw(&mut self, _w: &mut PistonWindow, _args: &RenderArgs) -> io::Result<i32> {
    Err(io::Error::from(io::ErrorKind::NotFound))
  }
}
