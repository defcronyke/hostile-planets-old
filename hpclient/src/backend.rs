use adapter_state::AdapterState;
use back;
use backend_state::BackendState;
use hal::Instance;
use window_state::WindowState;

#[cfg(feature = "gl")]
use hal::format::{AsFormat, Rgba8Srgb as ColorFormat};

#[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
pub fn create_backend(
  window_state: &mut WindowState,
) -> (BackendState<back::Backend>, back::Instance) {
  let window = window_state
    .wb
    .take()
    .unwrap()
    .build(&window_state.events_loop)
    .unwrap();
  let instance = back::Instance::create("gfx-rs quad", 1);
  let surface = instance.create_surface(&window);
  let mut adapters = instance.enumerate_adapters();
  (
    BackendState {
      adapter: AdapterState::new(&mut adapters),
      surface,
      window,
    },
    instance,
  )
}

#[cfg(feature = "gl")]
pub fn create_backend(window_state: &mut WindowState) -> (BackendState<back::Backend>, ()) {
  let window = {
    let builder =
      back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
        .with_vsync(true);
    back::glutin::GlWindow::new(
      window_state.wb.take().unwrap(),
      builder,
      &window_state.events_loop,
    ).unwrap()
  };

  let surface = back::Surface::from_window(window);
  let mut adapters = surface.enumerate_adapters();
  (
    BackendState {
      adapter: AdapterState::new(&mut adapters),
      surface,
    },
    (),
  )
}
