use desc_set::DescSet;
use device_state::DeviceState;
use hal::{pso, Backend, DescriptorPool, Device};
use std::cell::RefCell;
use std::rc::Rc;

pub struct DescSetLayout<B: Backend> {
  pub layout: Option<B::DescriptorSetLayout>,
  pub device: Rc<RefCell<DeviceState<B>>>,
}

impl<B: Backend> DescSetLayout<B> {
  pub fn new(
    device: Rc<RefCell<DeviceState<B>>>,
    bindings: Vec<pso::DescriptorSetLayoutBinding>,
  ) -> Self {
    let desc_set_layout = device
      .borrow()
      .device
      .create_descriptor_set_layout(bindings, &[]);

    DescSetLayout {
      layout: Some(desc_set_layout),
      device,
    }
  }

  pub fn create_desc_set(self, desc_pool: &mut B::DescriptorPool) -> DescSet<B> {
    let desc_set = desc_pool
      .allocate_set(self.layout.as_ref().unwrap())
      .unwrap();
    DescSet {
      layout: self,
      set: Some(desc_set),
    }
  }
}

impl<B: Backend> Drop for DescSetLayout<B> {
  fn drop(&mut self) {
    let device = &self.device.borrow().device;
    device.destroy_descriptor_set_layout(self.layout.take().unwrap());
  }
}
