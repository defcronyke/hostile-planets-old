use adapter_state::AdapterState;
use buffer_state::BufferState;
use color_range::COLOR_RANGE;
use desc_set::{DescSet, DescSetWrite};
use device_state::DeviceState;
use hal::format::{AsFormat, Rgba8Srgb as ColorFormat, Swizzle};
use hal::pso::PipelineStage;
use hal::queue::submission::Submission;
use hal::{buffer, command, format as f, image as i, memory as m, pso, Backend, Device};
use std::rc::Rc;

pub struct ImageState<B: Backend> {
  pub desc: DescSet<B>,
  pub buffer: Option<BufferState<B>>,
  pub sampler: Option<B::Sampler>,
  pub image_view: Option<B::ImageView>,
  pub image: Option<B::Image>,
  pub memory: Option<B::Memory>,
  pub transfered_image_fence: Option<B::Fence>,
}

impl<B: Backend> ImageState<B> {
  pub fn new<T: ::hal::Supports<::hal::Transfer>>(
    mut desc: DescSet<B>,
    img: &::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>,
    adapter: &AdapterState<B>,
    usage: buffer::Usage,
    device_state: &mut DeviceState<B>,
    staging_pool: &mut ::hal::CommandPool<B, ::hal::Graphics>,
  ) -> Self {
    let (buffer, dims, row_pitch, stride) = BufferState::new_texture(
      Rc::clone(&desc.layout.device),
      &mut device_state.device,
      img,
      adapter,
      usage,
    );

    let buffer = Some(buffer);
    let device = &mut device_state.device;

    let kind = i::Kind::D2(dims.width as i::Size, dims.height as i::Size, 1, 1);
    let unbound = device
      .create_image(
        kind,
        1,
        ColorFormat::SELF,
        i::Tiling::Optimal,
        i::Usage::TRANSFER_DST | i::Usage::SAMPLED,
        i::StorageFlags::empty(),
      )
      .unwrap(); // TODO: usage
    let req = device.get_image_requirements(&unbound);

    let device_type = adapter
      .memory_types
      .iter()
      .enumerate()
      .position(|(id, memory_type)| {
        req.type_mask & (1 << id) != 0
          && memory_type.properties.contains(m::Properties::DEVICE_LOCAL)
      })
      .unwrap()
      .into();

    let memory = device.allocate_memory(device_type, req.size).unwrap();

    let image = device.bind_image_memory(&memory, 0, unbound).unwrap();
    let image_view = device
      .create_image_view(
        &image,
        i::ViewKind::D2,
        ColorFormat::SELF,
        Swizzle::NO,
        COLOR_RANGE.clone(),
      )
      .unwrap();

    let sampler = device.create_sampler(i::SamplerInfo::new(i::Filter::Linear, i::WrapMode::Clamp));

    desc.write_to_state(
      vec![
        DescSetWrite {
          binding: 0,
          array_offset: 0,
          descriptors: Some(pso::Descriptor::Image(&image_view, i::Layout::Undefined)),
        },
        DescSetWrite {
          binding: 1,
          array_offset: 0,
          descriptors: Some(pso::Descriptor::Sampler(&sampler)),
        },
      ],
      device,
    );

    let mut transfered_image_fence = device.create_fence(false);

    // copy buffer to texture
    {
      let submit = {
        let mut cmd_buffer = staging_pool.acquire_command_buffer(false);

        let image_barrier = m::Barrier::Image {
          states: (i::Access::empty(), i::Layout::Undefined)
            ..(i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal),
          target: &image,
          range: COLOR_RANGE.clone(),
        };

        cmd_buffer.pipeline_barrier(
          PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
          m::Dependencies::empty(),
          &[image_barrier],
        );

        cmd_buffer.copy_buffer_to_image(
          buffer.as_ref().unwrap().get_buffer(),
          &image,
          i::Layout::TransferDstOptimal,
          &[command::BufferImageCopy {
            buffer_offset: 0,
            buffer_width: row_pitch / (stride as u32),
            buffer_height: dims.height as u32,
            image_layers: i::SubresourceLayers {
              aspects: f::Aspects::COLOR,
              level: 0,
              layers: 0..1,
            },
            image_offset: i::Offset { x: 0, y: 0, z: 0 },
            image_extent: i::Extent {
              width: dims.width,
              height: dims.height,
              depth: 1,
            },
          }],
        );

        let image_barrier = m::Barrier::Image {
          states: (i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal)
            ..(i::Access::SHADER_READ, i::Layout::ShaderReadOnlyOptimal),
          target: &image,
          range: COLOR_RANGE.clone(),
        };
        cmd_buffer.pipeline_barrier(
          PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
          m::Dependencies::empty(),
          &[image_barrier],
        );

        cmd_buffer.finish()
      };

      let submission = Submission::new().submit(Some(submit));
      device_state.queues.queues[0].submit(submission, Some(&mut transfered_image_fence));
    }

    ImageState {
      desc: desc,
      buffer: buffer,
      sampler: Some(sampler),
      image_view: Some(image_view),
      image: Some(image),
      memory: Some(memory),
      transfered_image_fence: Some(transfered_image_fence),
    }
  }

  pub fn wait_for_transfer_completion(&self) {
    let device = &self.desc.layout.device.borrow().device;
    device.wait_for_fence(self.transfered_image_fence.as_ref().unwrap(), !0);
  }

  pub fn get_layout(&self) -> &B::DescriptorSetLayout {
    self.desc.get_layout()
  }
}

impl<B: Backend> Drop for ImageState<B> {
  fn drop(&mut self) {
    {
      let device = &self.desc.layout.device.borrow().device;

      let fence = self.transfered_image_fence.take().unwrap();
      device.wait_for_fence(&fence, !0);
      device.destroy_fence(fence);

      device.destroy_sampler(self.sampler.take().unwrap());
      device.destroy_image_view(self.image_view.take().unwrap());
      device.destroy_image(self.image.take().unwrap());
      device.free_memory(self.memory.take().unwrap());
    }

    self.buffer.take().unwrap();
  }
}
