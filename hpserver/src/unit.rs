use std::collections::HashMap;
use std::vec::Vec;
use std::fmt::Debug;
use std::marker::Send;
use cpython::{Python, ToPyObject, PyDict};
use std::clone::Clone;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum UnitType {
  Scout,
}

impl ToPyObject for UnitType {
  type ObjectType = PyDict;

  fn to_py_object(&self, py: Python) -> PyDict {
    match self {
      UnitType::Scout => {
        let dict = PyDict::new(py);
        // dict.set_item(py, "server", self.server.clone()).unwrap(); // TODO
        dict
      }
    }
  }
}

pub trait Unit: Debug + Send + Sync {
  // Move x steps in the x direction, and y steps in the y direction.
  // Returns the world position of the unit after moving.
  fn go(&self, x: i64, y: i64) -> (i64, i64);

  // Get the world position of the unit.
  fn pos(&self) -> (i64, i64);

  fn box_clone(&self) -> Box<Unit>;
}

pub type Units = HashMap<UnitType, Vec<Box<Unit>>>;

impl Clone for Box<Unit> {
  fn clone(&self) -> Self {
    self.box_clone()
  }
}

impl ToPyObject for Box<Unit> {
  type ObjectType = PyDict;

  fn to_py_object(&self, py: Python) -> PyDict {
    PyDict::new(py)
  }
}
