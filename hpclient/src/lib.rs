#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cpython;
extern crate chrono;
extern crate timer;
extern crate toml;
extern crate piston_window;
extern crate vecmath;
extern crate camera_controllers;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate shader_version;
extern crate gltf;

pub mod client;
pub mod conf;
pub mod window;
pub mod cube;
pub mod vertex;
pub mod object;
pub mod asset_loader;
pub mod gltf_object;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
