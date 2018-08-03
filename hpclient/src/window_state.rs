use dims::DIMS;
use winit;

pub struct WindowState {
  pub events_loop: winit::EventsLoop,
  pub wb: Option<winit::WindowBuilder>,
}

impl WindowState {
  pub fn new() -> WindowState {
    let events_loop = winit::EventsLoop::new();

    let wb = winit::WindowBuilder::new()
      .with_dimensions(winit::dpi::LogicalSize::from_physical(
        winit::dpi::PhysicalSize {
          width: DIMS.width as _,
          height: DIMS.height as _,
        },
        1.0,
      ))
      .with_title("quad".to_string());

    WindowState {
      events_loop,
      wb: Some(wb),
    }
  }
}
