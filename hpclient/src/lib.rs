#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cpython;
extern crate piston_window;
extern crate chrono;
extern crate timer;
extern crate toml;

pub mod client;
pub mod conf;
pub mod window;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
