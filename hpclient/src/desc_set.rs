use desc_set_layout::DescSetLayout;
use hal::{pso, Backend, Device};
use std;

pub struct DescSet<B: Backend> {
  pub set: Option<B::DescriptorSet>,
  pub layout: DescSetLayout<B>,
}

pub struct DescSetWrite<W> {
  pub binding: pso::DescriptorBinding,
  pub array_offset: pso::DescriptorArrayIndex,
  pub descriptors: W,
}

impl<B: Backend> DescSet<B> {
  pub fn write_to_state<'a, 'b: 'a, W>(
    &'b mut self,
    write: Vec<DescSetWrite<W>>,
    device: &mut B::Device,
  ) where
    W: IntoIterator,
    W::Item: std::borrow::Borrow<pso::Descriptor<'a, B>>,
  {
    let set = self.set.as_ref().unwrap();
    let write: Vec<_> = write
      .into_iter()
      .map(|d| pso::DescriptorSetWrite {
        binding: d.binding,
        array_offset: d.array_offset,
        descriptors: d.descriptors,
        set,
      })
      .collect();
    device.write_descriptor_sets(write);
  }

  pub fn get_layout(&self) -> &B::DescriptorSetLayout {
    self.layout.layout.as_ref().unwrap()
  }
}
