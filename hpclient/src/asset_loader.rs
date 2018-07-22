use gltf_object::*;

use gltf;
use std::{fs, io};
use std::boxed::Box;
use std::error::Error as StdError;
use vecmath::*;
// use piston_window::*;

pub fn load_gltf(path: &str) -> Result<GltfObject, Box<StdError>> {
  let file = fs::File::open(&path)?;
  let reader = io::BufReader::new(file);
  let gltf_data = gltf::Gltf::from_reader(reader)?;

  let model = mat4_id();
  // let projection = GltfObject::get_projection(&w);
  let projection = mat4_id();

  println!("loaded gltf asset: {}", path);
  println!("gltf_data scenes: {:#?}", gltf_data.scenes());

  Ok(
    GltfObject {
      data: gltf_data,
      model: model,
      projection: projection,
    }
  )
}
