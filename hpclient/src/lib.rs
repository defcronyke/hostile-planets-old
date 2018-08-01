#![cfg_attr(
  not(
    any(
      feature = "vulkan",
      feature = "dx12",
      feature = "metal",
      feature = "gl"
    )
  ),
  allow(dead_code, unused_extern_crates, unused_imports)
)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cpython;
extern crate cgmath;
extern crate chrono;
extern crate env_logger;
extern crate timer;
extern crate toml;

#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(
  not(
    any(
      feature = "vulkan",
      feature = "dx12",
      feature = "metal",
      feature = "gl"
    )
  )
)]
extern crate gfx_backend_empty as back;
#[cfg(feature = "gl")]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

extern crate gfx_hal as hal;
extern crate glsl_to_spirv;
extern crate gltf;
extern crate image;
extern crate winit;

pub mod asset_loader;
pub mod client;
pub mod conf;
pub mod cube;
pub mod cube_data;
pub mod gltf_object;
pub mod object;
pub mod object_data;
pub mod quad;
pub mod vertex;
pub mod window;
pub mod window_data;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
