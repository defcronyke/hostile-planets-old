mod self::unit::*;

#[derive{Debug, Clone}]
struct Scout {
  type: UnitType::Scout,
  name: String,
  x: i64,
  y: i64,
}

pub impl Unit for Scout {
  fn new(x: i64, y: i64) -> Self {
    Scout{
      name: gen_unit_name(),
      x: 50,
      y: 50,
    }
  }

  fn go(&self, x: i64, y: i64) -> (i64, i64) {
    let newX = self.x + x;
    if newX < 0 {
      newX = 0;
    }

    let newY = self.y + y;
    if newY < 0 {
      newY = 0;
    }

    (newX, newY)
  }

  fn pos(&self) -> (i64, i64) {
    (self.x, self.y)
  }

  fn box_clone(&self) -> Box<Unit> {
    Box::new((*self).clone())
  }
}

impl ToPyObject for Scout {
    type ObjectType = PyDict;

    fn to_py_object(&self, py: Python) -> PyDict {
        let dict = PyDict::new(py);
        dict.set_item(py, "type", String::from("Scout")).unwrap();
        dict.set_item(py, "name", self.name.to_string()).unwrap();
        dict.set_item(py, "x", self.x).unwrap();
        dict.set_item(py, "y", self.y).unwrap();
        dict
    }
}
