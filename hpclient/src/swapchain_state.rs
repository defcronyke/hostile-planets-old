use backend_state::BackendState;
use device_state::DeviceState;
use hal;
use hal::format::ChannelType;
use hal::window::{Backbuffer, SwapchainConfig};
use hal::{format as f, image as i, Backend, Device, Surface};
use std::cell::RefCell;
use std::rc::Rc;
use surface_trait::SurfaceTrait;

pub struct SwapchainState<B: Backend>
where
  B::Surface: SurfaceTrait,
{
  pub swapchain: Option<B::Swapchain>,
  pub backbuffer: Option<Backbuffer<B>>,
  pub device: Rc<RefCell<DeviceState<B>>>,
  pub extent: hal::window::Extent2D,
  pub format: f::Format,
}

impl<B: Backend> SwapchainState<B>
where
  B::Surface: SurfaceTrait,
{
  pub fn new(backend: &mut BackendState<B>, device: Rc<RefCell<DeviceState<B>>>) -> Self {
    let (caps, formats, _present_modes) = backend
      .surface
      .compatibility(&device.borrow().physical_device);
    println!("formats: {:?}", formats);
    let format = formats.map_or(f::Format::Rgba8Srgb, |formats| {
      formats
        .iter()
        .find(|format| format.base_format().1 == ChannelType::Srgb)
        .map(|format| *format)
        .unwrap_or(formats[0])
    });

    let extent = match caps.current_extent {
      Some(e) => e,
      None => {
        #[cfg(not(feature = "gl"))]
        let window = &backend.window;
        #[cfg(feature = "gl")]
        let window = backend.surface.get_window_t();

        let window_size = window
          .get_inner_size()
          .unwrap()
          .to_physical(window.get_hidpi_factor());
        let mut extent = hal::window::Extent2D {
          width: window_size.width as _,
          height: window_size.height as _,
        };

        extent.width = extent
          .width
          .max(caps.extents.start.width)
          .min(caps.extents.end.width);
        extent.height = extent
          .height
          .max(caps.extents.start.height)
          .min(caps.extents.end.height);

        extent
      }
    };

    println!("Surface format: {:?}", format);

    let swap_config = SwapchainConfig::new()
      .with_color(format)
      .with_image_count(caps.image_count.start)
      .with_image_usage(i::Usage::COLOR_ATTACHMENT);
    let (swapchain, backbuffer) =
      device
        .borrow()
        .device
        .create_swapchain(&mut backend.surface, swap_config, None, &extent);

    let swapchain = SwapchainState {
      swapchain: Some(swapchain),
      backbuffer: Some(backbuffer),
      device,
      extent,
      format,
    };
    swapchain
  }
}

impl<B: Backend> Drop for SwapchainState<B>
where
  B::Surface: SurfaceTrait,
{
  fn drop(&mut self) {
    self
      .device
      .borrow()
      .device
      .destroy_swapchain(self.swapchain.take().unwrap());
  }
}
