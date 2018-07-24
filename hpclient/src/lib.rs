#![cfg_attr(
  not(any(feature = "vulkan", feature = "dx12", feature = "metal", feature = "gl")),
  allow(dead_code, unused_extern_crates, unused_imports)
)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cpython;
extern crate chrono;
extern crate timer;
extern crate toml;

extern crate env_logger;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "gl")]
extern crate gfx_backend_gl as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;

extern crate glsl_to_spirv;
extern crate image;
extern crate winit;
extern crate gltf;
// extern crate piston_window;
// extern crate vecmath;
// extern crate camera_controllers;
// #[macro_use]
// extern crate gfx;
// extern crate gfx_device_gl;
// extern crate shader_version;

pub mod client;
pub mod conf;
pub mod window;
pub mod cube;
pub mod vertex;
pub mod object;
pub mod gltf_object;
pub mod asset_loader;
pub mod quad;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
