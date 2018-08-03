use adapter_state::AdapterState;
use device_state::DeviceState;
use dimensions::Dimensions;
use hal::adapter::MemoryType;
use hal::{buffer, memory as m, Backend, Device};
use std::cell::RefCell;
use std::mem::size_of;
use std::rc::Rc;

pub struct BufferState<B: Backend> {
  pub memory: Option<B::Memory>,
  pub buffer: Option<B::Buffer>,
  pub device: Rc<RefCell<DeviceState<B>>>,
  pub size: u64,
}

impl<B: Backend> BufferState<B> {
  pub fn get_buffer(&self) -> &B::Buffer {
    self.buffer.as_ref().unwrap()
  }

  pub fn new<T>(
    device_ptr: Rc<RefCell<DeviceState<B>>>,
    data_source: &[T],
    usage: buffer::Usage,
    memory_types: &[MemoryType],
  ) -> Self
  where
    T: Copy,
  {
    let memory: B::Memory;
    let buffer: B::Buffer;
    let size: u64;

    let stride = size_of::<T>() as u64;
    let upload_size = data_source.len() as u64 * stride;

    {
      let device = &device_ptr.borrow().device;

      let unbound = device.create_buffer(upload_size, usage).unwrap();
      let mem_req = device.get_buffer_requirements(&unbound);

      // A note about performance: Using CPU_VISIBLE memory is convenient because it can be
      // directly memory mapped and easily updated by the CPU, but it is very slow and so should
      // only be used for small pieces of data that need to be updated very frequently. For something like
      // a vertex buffer that may be much larger and should not change frequently, you should instead
      // use a DEVICE_LOCAL buffer that gets filled by copying data from a CPU_VISIBLE staging buffer.
      let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
          mem_req.type_mask & (1 << id) != 0
            && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();

      memory = device.allocate_memory(upload_type, mem_req.size).unwrap();
      buffer = device.bind_buffer_memory(&memory, 0, unbound).unwrap();
      size = mem_req.size;

      // TODO: check transitions: read/write mapping and vertex buffer read
      {
        let mut data_target = device
          .acquire_mapping_writer::<T>(&memory, 0..size)
          .unwrap();
        data_target[0..data_source.len()].copy_from_slice(data_source);
        device.release_mapping_writer(data_target);
      }
    }

    BufferState {
      memory: Some(memory),
      buffer: Some(buffer),
      device: device_ptr,
      size,
    }
  }

  pub fn update_data<T>(&mut self, offset: u64, data_source: &[T])
  where
    T: Copy,
  {
    let device = &self.device.borrow().device;

    let stride = size_of::<T>() as u64;
    let upload_size = data_source.len() as u64 * stride;

    assert!(offset + upload_size <= self.size);

    let mut data_target = device
      .acquire_mapping_writer::<T>(self.memory.as_ref().unwrap(), offset..self.size)
      .unwrap();
    data_target[0..data_source.len()].copy_from_slice(data_source);
    device.release_mapping_writer(data_target);
  }

  pub fn new_texture(
    device_ptr: Rc<RefCell<DeviceState<B>>>,
    device: &mut B::Device,
    img: &::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>,
    adapter: &AdapterState<B>,
    usage: buffer::Usage,
  ) -> (Self, Dimensions<u32>, u32, usize) {
    let (width, height) = img.dimensions();

    let row_alignment_mask = adapter.limits.min_buffer_copy_pitch_alignment as u32 - 1;
    let stride = 4usize;

    let row_pitch = (width * stride as u32 + row_alignment_mask) & !row_alignment_mask;
    let upload_size = (height * row_pitch) as u64;

    let memory: B::Memory;
    let buffer: B::Buffer;
    let size: u64;

    {
      let unbound = device.create_buffer(upload_size, usage).unwrap();
      let mem_reqs = device.get_buffer_requirements(&unbound);

      let upload_type = adapter
        .memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
          mem_reqs.type_mask & (1 << id) != 0
            && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();

      memory = device.allocate_memory(upload_type, mem_reqs.size).unwrap();
      buffer = device.bind_buffer_memory(&memory, 0, unbound).unwrap();
      size = mem_reqs.size;

      // copy image data into staging buffer
      {
        let mut data_target = device
          .acquire_mapping_writer::<u8>(&memory, 0..size)
          .unwrap();

        for y in 0..height as usize {
          let data_source_slice =
            &(**img)[y * (width as usize) * stride..(y + 1) * (width as usize) * stride];
          let dest_base = y * row_pitch as usize;

          data_target[dest_base..dest_base + data_source_slice.len()]
            .copy_from_slice(data_source_slice);
        }

        device.release_mapping_writer(data_target);
      }
    }

    (
      BufferState {
        memory: Some(memory),
        buffer: Some(buffer),
        device: device_ptr,
        size,
      },
      Dimensions { width, height },
      row_pitch,
      stride,
    )
  }
}

impl<B: Backend> Drop for BufferState<B> {
  fn drop(&mut self) {
    let device = &self.device.borrow().device;
    device.destroy_buffer(self.buffer.take().unwrap());
    device.free_memory(self.memory.take().unwrap());
  }
}
