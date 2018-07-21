use piston_window::*;

pub trait Window {

}

pub struct _PistonWindow {
  pub window: PistonWindow,
  pub opengl: OpenGL,
}

impl Window for _PistonWindow {}

impl _PistonWindow {
  pub fn new() -> Self {
    let opengl = OpenGL::V3_2;  // Set OpenGL version here.

    let mut w: PistonWindow = WindowSettings::new("piston: cube", [640, 480])
      .exit_on_esc(true)
      .samples(4)
      .opengl(opengl)
      .build()
      .unwrap();
    
    w.set_capture_cursor(true);

    Self {
      window: w,
      opengl: opengl,
    }
  }
}
