use cgmath::Matrix4;

#[derive(Copy, Clone)]
pub struct UniformMatricesData<T> {
  pub model: Matrix4<T>,
  pub view: Matrix4<T>,
  pub proj: Matrix4<T>,
}

impl<T> UniformMatricesData<T> {
  pub fn new(model: Matrix4<T>, view: Matrix4<T>, proj: Matrix4<T>) -> Self {
    Self { model, view, proj }
  }
}
