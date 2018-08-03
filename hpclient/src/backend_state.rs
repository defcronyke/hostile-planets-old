use adapter_state::AdapterState;
use hal::Backend;

#[cfg(not(feature = "gl"))]
use window_type::WindowType;

pub struct BackendState<B: Backend> {
  pub surface: B::Surface,
  pub adapter: AdapterState<B>,
  #[cfg(not(feature = "gl"))]
  pub window: WindowType,
}
