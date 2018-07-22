use gltf_object::*;
use object::Object;

use gltf;
use std::{fs, io};
use std::boxed::Box;
use std::error::Error as StdError;
// use vecmath;
use vecmath::*;
use piston_window::*;

pub fn load_gltf(w: &mut PistonWindow, path: &str) -> Result<GltfObject, Box<StdError>> {
  let file = fs::File::open(&path)?;
  let reader = io::BufReader::new(file);
  let gltf_data = gltf::Gltf::from_reader(reader)?;
  // println!("{:#?}", gltf);

  let model = mat4_id();
  let projection = GltfObject::get_projection(&w);

  Ok(
    GltfObject {
      data: gltf_data,
      model: model,
      projection: projection,
    }
  )
}
