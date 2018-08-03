#[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
use winit;

#[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
pub type WindowType = winit::Window;
#[cfg(feature = "gl")]
pub type WindowType = u32;
