use window::*;

use std::io;

pub trait Object {
  fn get_name(&self) -> String {
    String::from("unknown")
  }

  fn draw(&self, _w: &mut _PistonWindow) -> io::Result<i32> {
    Err(io::Error::from(io::ErrorKind::NotFound))
  }
}
