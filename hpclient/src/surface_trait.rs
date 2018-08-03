use back;
use hal;

pub trait SurfaceTrait {
  #[cfg(feature = "gl")]
  fn get_window_t(&self) -> &back::glutin::GlWindow;
}

impl SurfaceTrait for <back::Backend as hal::Backend>::Surface {
  #[cfg(feature = "gl")]
  fn get_window_t(&self) -> &back::glutin::GlWindow {
    self.get_window()
  }
}
