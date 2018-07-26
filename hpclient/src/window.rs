use winit;
use back;
use hal;
use hal::{Instance, adapter};

#[cfg(feature = "gl")]
use hal::format::{AsFormat, Rgba8Srgb as ColorFormat};

pub trait Window {

}

#[cfg(not(feature = "gl"))]
pub struct _WinitWindow {
  pub window: winit::Window,
  pub instance: back::Instance,
  pub adapters: Vec<adapter::Adapter<back::Backend>>,
  pub surface: <back::Backend as hal::Backend>::Surface,
}

#[cfg(feature = "gl")]
pub struct _WinitWindow {
  pub adapters: Vec<adapter::Adapter<back::Backend>>,
  pub surface: <back::Backend as hal::Backend>::Surface,
}

impl Window for _WinitWindow {}

impl _WinitWindow {
  pub fn new(title: &str, w: u64, h: u64, events_loop: &winit::EventsLoop) -> Self {
    let wb = winit::WindowBuilder::new()
      .with_dimensions(winit::dpi::LogicalSize::from_physical(
        winit::dpi::PhysicalSize {
          width: w as _,
          height: h as _,
        },
        1.0,
      ))
      .with_title(title.to_string());

    // instantiate backend
    #[cfg(not(feature = "gl"))]
    let (window, instance, adapters, surface) = {
      let window = wb.build(&events_loop).unwrap();
      let instance = back::Instance::create(&format!("hp {}", title), 1);
      let adapters = instance.enumerate_adapters();
      let surface = instance.create_surface(&window);
      (window, instance, adapters, surface)
    };

    #[cfg(feature = "gl")]
    let (adapters, surface) = {
      let window = {
        let builder =
          back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
            .with_vsync(true);
        back::glutin::GlWindow::new(wb, builder, &events_loop).unwrap()
      };

      let surface = back::Surface::from_window(window);
      let adapters = surface.enumerate_adapters();
      (adapters, surface)
    };

    #[cfg(not(feature = "gl"))]
    return Self {
      window: window,
      // window_builder: wb,
      instance: instance,
      adapters: adapters,
      surface: surface,
    };

    #[cfg(feature = "gl")]
    return Self {
      // window_builder: wb,
      adapters: adapters,
      surface: surface,
    };
  }
}
