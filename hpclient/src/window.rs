use piston_window::*;

pub trait Window {

}

pub struct _PistonWindow {
  pub window: PistonWindow,
}

impl Window for _PistonWindow {}

impl _PistonWindow {
  pub fn new() -> Self {
    Self {
      window: WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap(),
    }
  }
}
