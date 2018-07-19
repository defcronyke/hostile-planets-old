use unit::*;

use cpython::{Python, ToPyObject, PyDict};
use std::clone::Clone;

#[derive(Debug, Clone)]
pub struct _Player {
  pub name:   String,
  pub units:  Units,
}

impl _Player {
  pub fn new(name: &str, units: Units) -> Self {
    _Player {
      name: String::from(name),
      units: units,
    }
  }
}

impl ToPyObject for _Player {
    type ObjectType = PyDict;

    fn to_py_object(&self, py: Python) -> PyDict {
        let dict = PyDict::new(py);
        dict.set_item(py, "name", self.name.clone()).unwrap();
        dict.set_item(py, "units", self.units.clone()).unwrap();
        dict
    }
}
