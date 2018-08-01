use back;
use back::Backend;
use cgmath::{Matrix4, SquareMatrix};
use cube_data::_CubeData;
use hal;
use hal::format::{AsFormat, Rgba8Srgb as ColorFormat, Swizzle};
use hal::pso::PipelineStage;
use hal::queue::Submission;
use hal::{
  buffer, command, format as f, image as i, memory as m, pso, CommandPool, Device, Graphics,
  IndexType, Limits, QueueGroup, Swapchain,
};
use image;
use object::Object;
use std;
use std::io::Cursor;
use vertex::Vertex;
use window_data::_WinitWindowData;

const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
  aspects: f::Aspects::COLOR,
  levels: 0..1,
  layers: 0..1,
};

pub struct Cube {
  pub vertices: [Vertex; 24],
  pub indices: [u16; 36],
  pub texels: [[u8; 4]; 4],
  pub model_matrix: Matrix4<f32>,
}

impl Cube {
  pub fn new() -> Self {
    Self {
      vertices: CUBE_VERTICES,
      indices: CUBE_INDICES,
      texels: CUBE_TEXELS,
      model_matrix: Matrix4::identity(),
    }
  }

  pub fn init(
    &self,
    device: &back::Device,
    memory_types: &Vec<hal::MemoryType>,
    limits: &Limits,
    desc_set: &<Backend as hal::Backend>::DescriptorSet,
    command_pool: &mut CommandPool<Backend, Graphics>,
    queue_group: &mut QueueGroup<Backend, Graphics>,
  ) -> _CubeData {
    let buffer_stride = std::mem::size_of::<Vertex>() as u64;
    let buffer_len = self.vertices.len() as u64 * buffer_stride;

    let buffer_unbound = device
      .create_buffer(buffer_len, buffer::Usage::VERTEX)
      .unwrap();
    let buffer_req = device.get_buffer_requirements(&buffer_unbound);

    let upload_type = memory_types
      .iter()
      .enumerate()
      .position(|(id, mem_type)| {
        buffer_req.type_mask & (1 << id) != 0
          && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
      })
      .unwrap()
      .into();

    let buffer_memory = device
      .allocate_memory(upload_type, buffer_req.size)
      .unwrap();
    let vertex_buffer = device
      .bind_buffer_memory(&buffer_memory, 0, buffer_unbound)
      .unwrap();

    // TODO: check transitions: read/write mapping and vertex buffer read
    {
      let mut vertices = device
        .acquire_mapping_writer::<Vertex>(&buffer_memory, 0..buffer_len)
        .unwrap();
      vertices.copy_from_slice(&self.vertices);
      device.release_mapping_writer(vertices);
    }

    let index_buffer_stride = std::mem::size_of::<u16>() as u64;
    let index_buffer_len = self.indices.len() as u64 * index_buffer_stride;

    let index_buffer_unbound = device
      .create_buffer(buffer_len, buffer::Usage::INDEX)
      .unwrap();
    let index_buffer_req = device.get_buffer_requirements(&index_buffer_unbound);

    let index_upload_type: hal::MemoryTypeId = memory_types
      .iter()
      .enumerate()
      .position(|(id, mem_type)| {
        index_buffer_req.type_mask & (1 << id) != 0
          && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
      })
      .unwrap()
      .into();

    let index_buffer_memory = device
      .allocate_memory(index_upload_type, index_buffer_req.size)
      .unwrap();
    let index_buffer = device
      .bind_buffer_memory(&index_buffer_memory, 0, index_buffer_unbound)
      .unwrap();

    // TODO: check transitions: read/write mapping and vertex buffer read
    {
      let mut indices = device
        .acquire_mapping_writer::<u16>(&index_buffer_memory, 0..index_buffer_len)
        .unwrap();
      indices.copy_from_slice(&self.indices);
      device.release_mapping_writer(indices);
    }

    // Image
    let img_data = include_bytes!("../../data/logo.png");

    let img = image::load(Cursor::new(&img_data[..]), image::PNG)
      .unwrap()
      .to_rgba();
    let (width, height) = img.dimensions();
    let kind = i::Kind::D2(width as i::Size, height as i::Size, 1, 1);
    let row_alignment_mask = limits.min_buffer_copy_pitch_alignment as u32 - 1;
    let image_stride = 4usize;
    let row_pitch = (width * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
    let upload_size = (height * row_pitch) as u64;

    let image_buffer_unbound = device
      .create_buffer(upload_size, buffer::Usage::TRANSFER_SRC)
      .unwrap();
    let image_mem_reqs = device.get_buffer_requirements(&image_buffer_unbound);
    let image_upload_memory = device
      .allocate_memory(upload_type, image_mem_reqs.size)
      .unwrap();
    let image_upload_buffer = device
      .bind_buffer_memory(&image_upload_memory, 0, image_buffer_unbound)
      .unwrap();

    // copy image data into staging buffer
    {
      let mut data = device
        .acquire_mapping_writer::<u8>(&image_upload_memory, 0..upload_size)
        .unwrap();
      for y in 0..height as usize {
        let row =
          &(*img)[y * (width as usize) * image_stride..(y + 1) * (width as usize) * image_stride];
        let dest_base = y * row_pitch as usize;
        data[dest_base..dest_base + row.len()].copy_from_slice(row);
      }
      device.release_mapping_writer(data);
    }

    let image_unbound = device
      .create_image(
        kind,
        1,
        ColorFormat::SELF,
        i::Tiling::Optimal,
        i::Usage::TRANSFER_DST | i::Usage::SAMPLED,
        i::StorageFlags::empty(),
      )
      .unwrap(); // TODO: usage
    let image_req = device.get_image_requirements(&image_unbound);

    let device_type = memory_types
      .iter()
      .enumerate()
      .position(|(id, memory_type)| {
        image_req.type_mask & (1 << id) != 0
          && memory_type.properties.contains(m::Properties::DEVICE_LOCAL)
      })
      .unwrap()
      .into();
    let image_memory = device.allocate_memory(device_type, image_req.size).unwrap();

    let image_logo = device
      .bind_image_memory(&image_memory, 0, image_unbound)
      .unwrap();
    let image_srv = device
      .create_image_view(
        &image_logo,
        i::ViewKind::D2,
        ColorFormat::SELF,
        Swizzle::NO,
        COLOR_RANGE.clone(),
      )
      .unwrap();

    let sampler = device.create_sampler(i::SamplerInfo::new(i::Filter::Linear, i::WrapMode::Clamp));

    device.write_descriptor_sets(vec![
      pso::DescriptorSetWrite {
        set: desc_set,
        binding: 0,
        array_offset: 0,
        descriptors: Some(pso::Descriptor::Image(&image_srv, i::Layout::Undefined)),
      },
      pso::DescriptorSetWrite {
        set: desc_set,
        binding: 1,
        array_offset: 0,
        descriptors: Some(pso::Descriptor::Sampler(&sampler)),
      },
    ]);

    let frame_semaphore = device.create_semaphore();
    let mut frame_fence = device.create_fence(false); // TODO: remove

    // copy buffer to texture
    {
      let submit = {
        let mut cmd_buffer = command_pool.acquire_command_buffer(false);

        let image_barrier = m::Barrier::Image {
          states: (i::Access::empty(), i::Layout::Undefined)
            ..(i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal),
          target: &image_logo,
          range: COLOR_RANGE.clone(),
        };

        cmd_buffer.pipeline_barrier(
          PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
          m::Dependencies::empty(),
          &[image_barrier],
        );

        cmd_buffer.copy_buffer_to_image(
          &image_upload_buffer,
          &image_logo,
          i::Layout::TransferDstOptimal,
          &[command::BufferImageCopy {
            buffer_offset: 0,
            buffer_width: row_pitch / (image_stride as u32),
            buffer_height: height as u32,
            image_layers: i::SubresourceLayers {
              aspects: f::Aspects::COLOR,
              level: 0,
              layers: 0..1,
            },
            image_offset: i::Offset { x: 0, y: 0, z: 0 },
            image_extent: i::Extent {
              width,
              height,
              depth: 1,
            },
          }],
        );

        let image_barrier = m::Barrier::Image {
          states: (i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal)
            ..(i::Access::SHADER_READ, i::Layout::ShaderReadOnlyOptimal),
          target: &image_logo,
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
      queue_group.queues[0].submit(submission, Some(&mut frame_fence));

      device.wait_for_fence(&frame_fence, !0);
    }

    _CubeData {
      vertex_buffer,
      index_buffer,
      image_upload_buffer,
      image_logo,
      image_srv,
      sampler,
      frame_fence,
      frame_semaphore,
      buffer_memory,
      index_buffer_memory,
      image_memory,
      image_upload_memory,
    }
  }

  pub fn render(
    &self,
    data: &mut _WinitWindowData,
    cube_data: &mut _CubeData,
    frame: u32,
    recreate_swapchain: bool,
  ) -> bool {
    // Rendering
    let mut recreate_swapchain = recreate_swapchain;
    let submit = {
      let mut cmd_buffer = data.command_pool.acquire_command_buffer(false);

      cmd_buffer.set_viewports(0, &[data.viewport.clone()]);
      cmd_buffer.set_scissors(0, &[data.viewport.rect]);
      cmd_buffer.bind_graphics_pipeline(&data.pipeline);
      cmd_buffer.bind_vertex_buffers(0, Some((&cube_data.vertex_buffer, 0)));
      cmd_buffer.bind_index_buffer(buffer::IndexBufferView {
        buffer: &cube_data.index_buffer,
        offset: 0,
        index_type: IndexType::U16,
      });
      cmd_buffer.bind_graphics_descriptor_sets(&data.pipeline_layout, 0, Some(&data.desc_set), &[]); //TODO

      {
        let mut encoder = cmd_buffer.begin_render_pass_inline(
          &data.render_pass,
          &data.framebuffers[frame as usize],
          data.viewport.rect,
          &[command::ClearValue::Color(command::ClearColor::Float([
            0.8, 0.8, 0.8, 1.0,
          ]))],
        );

        // TODO: Last argument is range of instances. What are instances?
        encoder.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
      }

      cmd_buffer.finish()
    };

    let submission = Submission::new()
      .wait_on(&[(&cube_data.frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)])
      .submit(Some(submit));
    data.queue_group.queues[0].submit(submission, Some(&mut cube_data.frame_fence));

    // TODO: replace with semaphore
    data.device.wait_for_fence(&cube_data.frame_fence, !0);

    // present frame
    if let Err(_) = data
      .swap_chain
      .present(&mut data.queue_group.queues[0], frame, &[])
    {
      recreate_swapchain = true;
    }

    recreate_swapchain
  }

  pub fn cleanup(device: &back::Device, data: _CubeData) {
    device.destroy_buffer(data.vertex_buffer);
    device.destroy_buffer(data.index_buffer);
    device.destroy_buffer(data.image_upload_buffer);
    device.destroy_image(data.image_logo);
    device.destroy_image_view(data.image_srv);
    device.destroy_sampler(data.sampler);
    device.destroy_fence(data.frame_fence);
    device.destroy_semaphore(data.frame_semaphore);
    device.free_memory(data.buffer_memory);
    device.free_memory(data.index_buffer_memory);
    device.free_memory(data.image_memory);
    device.free_memory(data.image_upload_memory);
  }
}

impl Object for Cube {}

pub const CUBE_VERTICES: [Vertex; 24] = [
  //top (0, 0, 1)
  Vertex {
    a_Pos: [-1.0, -1.0, 1.0],
    a_Uv: [0.0, 0.0],
  },
  Vertex {
    a_Pos: [1.0, -1.0, 1.0],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [1.0, 1.0, 1.0],
    a_Uv: [1.0, 1.0],
  },
  Vertex {
    a_Pos: [-1.0, 1.0, 1.0],
    a_Uv: [0.0, 1.0],
  },
  //bottom (0, 0, -1)
  Vertex {
    a_Pos: [1.0, 1.0, -1.0],
    a_Uv: [0.0, 0.0],
  },
  Vertex {
    a_Pos: [-1.0, 1.0, -1.0],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [-1.0, -1.0, -1.0],
    a_Uv: [1.0, 1.0],
  },
  Vertex {
    a_Pos: [1.0, -1.0, -1.0],
    a_Uv: [0.0, 1.0],
  },
  //right (1, 0, 0)
  Vertex {
    a_Pos: [1.0, -1.0, -1.0],
    a_Uv: [0.0, 0.0],
  },
  Vertex {
    a_Pos: [1.0, 1.0, -1.0],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [1.0, 1.0, 1.0],
    a_Uv: [1.0, 1.0],
  },
  Vertex {
    a_Pos: [1.0, -1.0, 1.0],
    a_Uv: [0.0, 1.0],
  },
  //left (-1, 0, 0)
  Vertex {
    a_Pos: [-1.0, 1.0, 1.0],
    a_Uv: [0.0, 0.0],
  },
  Vertex {
    a_Pos: [-1.0, -1.0, 1.0],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [-1.0, -1.0, -1.0],
    a_Uv: [1.0, 1.0],
  },
  Vertex {
    a_Pos: [-1.0, 1.0, -1.0],
    a_Uv: [0.0, 1.0],
  },
  //front (0, 1, 0)
  Vertex {
    a_Pos: [-1.0, 1.0, -1.0],
    a_Uv: [0.0, 0.0],
  },
  Vertex {
    a_Pos: [1.0, 1.0, -1.0],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [1.0, 1.0, 1.0],
    a_Uv: [1.0, 1.0],
  },
  Vertex {
    a_Pos: [-1.0, 1.0, 1.0],
    a_Uv: [0.0, 1.0],
  },
  //back (0, -1, 0)
  Vertex {
    a_Pos: [1.0, -1.0, 1.0],
    a_Uv: [0.0, 0.0],
  },
  Vertex {
    a_Pos: [-1.0, -1.0, 1.0],
    a_Uv: [1.0, 0.0],
  },
  Vertex {
    a_Pos: [-1.0, -1.0, -1.0],
    a_Uv: [1.0, 1.0],
  },
  Vertex {
    a_Pos: [1.0, -1.0, -1.0],
    a_Uv: [0.0, 1.0],
  },
];

const CUBE_INDICES: [u16; 36] = [
  0, 1, 2, 2, 3, 0, // top
  4, 6, 5, 6, 4, 7, // bottom
  8, 9, 10, 10, 11, 8, // right
  12, 14, 13, 14, 12, 15, // left
  16, 18, 17, 18, 16, 19, // front
  20, 21, 22, 22, 23, 20, // back
];

const CUBE_TEXELS: [[u8; 4]; 4] = [
  [0xff, 0xff, 0xff, 0x00],
  [0xff, 0x00, 0x00, 0x00],
  [0x00, 0xff, 0x00, 0x00],
  [0x00, 0x00, 0xff, 0x00],
];
