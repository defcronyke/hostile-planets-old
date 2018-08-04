pub trait Object: Send + Sync {
  fn get_name(&self) -> String {
    String::from("an unknown object")
  }
}
