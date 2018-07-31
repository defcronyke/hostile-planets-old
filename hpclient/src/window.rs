use back;
use back::Backend;
use cube::Cube;
use glsl_to_spirv;
use hal;
use hal::format::{AsFormat, ChannelType, Rgba8Srgb as ColorFormat, Swizzle};
use hal::pass::Subpass;
use hal::pso::{PipelineStage, ShaderStageFlags, Specialization, Viewport};
use hal::queue::Submission;
use hal::window::Extent2D;
use hal::{
  adapter, buffer, command, format as f, image as i, memory as m, pass, pool, pso, Adapter,
  Backbuffer, CommandPool, DescriptorPool, Device, Graphics, Instance, PhysicalDevice, Primitive,
  QueueGroup, Surface, SwapchainConfig,
};
use image;
// use quad::Quad;
use std;
use std::fs;
use std::io::{Cursor, Read};
use vertex::Vertex;
use winit;

const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
  aspects: f::Aspects::COLOR,
  levels: 0..1,
  layers: 0..1,
};

const ENTRY_NAME: &str = "main";

#[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
type WindowType = winit::Window;
#[cfg(feature = "gl")]
type WindowType = u32;

pub struct _WinitWindowData {
  pub device: back::Device,
  pub command_pool: CommandPool<Backend, Graphics>,
  pub frame_semaphore: <Backend as hal::Backend>::Semaphore,
  pub frame_fence: <Backend as hal::Backend>::Fence,
  pub pipeline: <Backend as hal::Backend>::GraphicsPipeline,
  pub pipeline_layout: <Backend as hal::Backend>::PipelineLayout,
  pub framebuffers: Vec<<Backend as hal::Backend>::Framebuffer>,
  pub frame_images: Vec<(
    <Backend as hal::Backend>::Image,
    <Backend as hal::Backend>::ImageView,
  )>,
  pub render_pass: <Backend as hal::Backend>::RenderPass,
  pub swap_chain: <Backend as hal::Backend>::Swapchain,
  pub adapter: Adapter<Backend>,
  pub set_layout: <Backend as hal::Backend>::DescriptorSetLayout,
  pub extent: Extent2D,
  pub viewport: Viewport,
  pub vertex_buffer: <Backend as hal::Backend>::Buffer,
  pub buffer_memory: <Backend as hal::Backend>::Memory,
  pub index_buffer: <Backend as hal::Backend>::Buffer,
  pub index_buffer_memory: <Backend as hal::Backend>::Memory,
  pub desc_set: <Backend as hal::Backend>::DescriptorSet,
  pub desc_pool: <Backend as hal::Backend>::DescriptorPool,
  pub queue_group: QueueGroup<Backend, Graphics>,
  pub image_upload_buffer: <Backend as hal::Backend>::Buffer,
  pub image_logo: <Backend as hal::Backend>::Image,
  pub image_srv: <Backend as hal::Backend>::ImageView,
  pub sampler: <Backend as hal::Backend>::Sampler,
  pub image_memory: <Backend as hal::Backend>::Memory,
  pub image_upload_memory: <Backend as hal::Backend>::Memory,
}

pub trait Window {}

#[cfg(not(feature = "gl"))]
pub struct _WinitWindow {
  pub window: WindowType,
  pub instance: back::Instance,
  pub adapters: Vec<adapter::Adapter<back::Backend>>,
  pub surface: <back::Backend as hal::Backend>::Surface,
}

#[cfg(feature = "gl")]
pub struct _WinitWindow {
  pub window: WindowType,
  pub adapters: Vec<adapter::Adapter<back::Backend>>,
  pub surface: <back::Backend as hal::Backend>::Surface,
}

impl Window for _WinitWindow {}

impl _WinitWindow {
  pub fn new(title: &str, w: u64, h: u64, events_loop: &winit::EventsLoop) -> Self {
    let wb = winit::WindowBuilder::new()
      .with_dimensions(winit::dpi::LogicalSize::from_physical(
        winit::dpi::PhysicalSize {
          width: w as _,
          height: h as _,
        },
        1.0,
      ))
      .with_title(title.to_string());

    // instantiate backend
    #[cfg(not(feature = "gl"))]
    let (window, instance, adapters, surface) = {
      let window = wb.build(&events_loop).unwrap();
      let instance = back::Instance::create(&format!("hp {}", title), 1);
      let adapters = instance.enumerate_adapters();
      let surface = instance.create_surface(&window);
      (window, instance, adapters, surface)
    };

    #[cfg(feature = "gl")]
    let (adapters, surface) = {
      let window = {
        let builder =
          back::config_context(back::glutin::ContextBuilder::new(), ColorFormat::SELF, None)
            .with_vsync(true);
        back::glutin::GlWindow::new(wb, builder, &events_loop).unwrap()
      };

      let surface = back::Surface::from_window(window);
      let adapters = surface.enumerate_adapters();
      (adapters, surface)
    };

    #[cfg(not(feature = "gl"))]
    return Self {
      window: window,
      instance: instance,
      adapters: adapters,
      surface: surface,
    };

    #[cfg(feature = "gl")]
    return Self {
      window: 0,
      adapters: adapters,
      surface: surface,
    };
  }

  pub fn init(&mut self) -> _WinitWindowData {
    for adapter in &self.adapters {
      println!("{:?}", adapter.info);
    }

    let mut adapter = self.adapters.remove(0);
    let memory_types = adapter.physical_device.memory_properties().memory_types;
    let limits = adapter.physical_device.limits();

    // Build a new device and associated command queues
    let (mut device, mut queue_group) = adapter
      .open_with::<_, hal::Graphics>(1, |family| self.surface.supports_queue_family(family))
      .unwrap();

    let mut command_pool =
      device.create_command_pool_typed(&queue_group, pool::CommandPoolCreateFlags::empty(), 16);

    // Setup renderpass and pipeline
    let set_layout = device.create_descriptor_set_layout(
      &[
        pso::DescriptorSetLayoutBinding {
          binding: 0,
          ty: pso::DescriptorType::SampledImage,
          count: 1,
          stage_flags: ShaderStageFlags::FRAGMENT,
          immutable_samplers: false,
        },
        pso::DescriptorSetLayoutBinding {
          binding: 1,
          ty: pso::DescriptorType::Sampler,
          count: 1,
          stage_flags: ShaderStageFlags::FRAGMENT,
          immutable_samplers: false,
        },
      ],
      &[],
    );

    // Descriptors
    let mut desc_pool = device.create_descriptor_pool(
      1, // sets
      &[
        pso::DescriptorRangeDesc {
          ty: pso::DescriptorType::SampledImage,
          count: 1,
        },
        pso::DescriptorRangeDesc {
          ty: pso::DescriptorType::Sampler,
          count: 1,
        },
      ],
    );
    let desc_set = desc_pool.allocate_set(&set_layout).unwrap();

    // Buffer allocations
    println!("Memory types: {:?}", memory_types);

    let cube = Cube::new();
    let buffer_stride = std::mem::size_of::<Vertex>() as u64;
    let buffer_len = cube.vertices.len() as u64 * buffer_stride;
    // let quad = Quad::new();
    // let buffer_stride = std::mem::size_of::<Vertex2D>() as u64;
    // let buffer_len = quad.vertices.len() as u64 * buffer_stride;

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
      vertices.copy_from_slice(&cube.vertices);
      device.release_mapping_writer(vertices);
    }

    let index_buffer_stride = std::mem::size_of::<u16>() as u64;
    let index_buffer_len = cube.indices.len() as u64 * index_buffer_stride;

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
      indices.copy_from_slice(&cube.indices);
      device.release_mapping_writer(indices);
    }

    // Image
    let img_data = include_bytes!("../../data/logo.png");
    // let img_data = include_bytes!("../../../gfx/examples/quad/data/logo.png");

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
        set: &desc_set,
        binding: 0,
        array_offset: 0,
        descriptors: Some(pso::Descriptor::Image(&image_srv, i::Layout::Undefined)),
      },
      pso::DescriptorSetWrite {
        set: &desc_set,
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

    #[cfg(not(feature = "gl"))]
    let window = &self.window;
    #[cfg(feature = "gl")]
    let window = &0;
    let (
      new_swap_chain,
      new_render_pass,
      new_framebuffers,
      new_frame_images,
      new_pipeline,
      new_pipeline_layout,
      new_extent,
    ) = Self::swapchain_stuff(
      &mut self.surface,
      &mut device,
      &adapter.physical_device,
      &set_layout,
      window,
    );

    // Rendering setup
    let viewport = pso::Viewport {
      rect: pso::Rect {
        x: 0,
        y: 0,
        w: new_extent.width as _,
        h: new_extent.height as _,
      },
      depth: 0.0..1.0,
    };

    _WinitWindowData {
      swap_chain: new_swap_chain,
      render_pass: new_render_pass,
      framebuffers: new_framebuffers,
      frame_images: new_frame_images,
      pipeline: new_pipeline,
      pipeline_layout: new_pipeline_layout,
      extent: new_extent,
      device,
      command_pool,
      frame_semaphore,
      frame_fence,
      adapter,
      set_layout,
      viewport,
      vertex_buffer,
      buffer_memory,
      index_buffer,
      index_buffer_memory,
      desc_set,
      desc_pool,
      queue_group,
      image_upload_buffer,
      image_logo,
      image_srv,
      sampler,
      image_memory,
      image_upload_memory,
    }
  }

  pub fn recreate_swapchain(
    _w: &WindowType,
    s: &mut <back::Backend as hal::Backend>::Surface,
    data: _WinitWindowData,
  ) -> _WinitWindowData {
    let mut data = data;
    data.device.wait_idle().unwrap();

    data.command_pool.reset();

    data.device.destroy_graphics_pipeline(data.pipeline);
    data.device.destroy_pipeline_layout(data.pipeline_layout);

    for framebuffer in data.framebuffers {
      data.device.destroy_framebuffer(framebuffer);
    }

    for (_, rtv) in data.frame_images {
      data.device.destroy_image_view(rtv);
    }
    data.device.destroy_render_pass(data.render_pass);
    data.device.destroy_swapchain(data.swap_chain);

    #[cfg(not(feature = "gl"))]
    let window = _w;
    #[cfg(feature = "gl")]
    let window = &0;
    let (
      new_swap_chain,
      new_render_pass,
      new_framebuffers,
      new_frame_images,
      new_pipeline,
      new_pipeline_layout,
      new_extent,
    ) = Self::swapchain_stuff(
      s,
      &mut data.device,
      &data.adapter.physical_device,
      &data.set_layout,
      window,
    );

    data.viewport.rect.w = data.extent.width as _;
    data.viewport.rect.h = data.extent.height as _;

    _WinitWindowData {
      pipeline: new_pipeline,
      pipeline_layout: new_pipeline_layout,
      framebuffers: new_framebuffers,
      frame_images: new_frame_images,
      render_pass: new_render_pass,
      swap_chain: new_swap_chain,
      extent: new_extent,
      device: data.device,
      command_pool: data.command_pool,
      frame_semaphore: data.frame_semaphore,
      frame_fence: data.frame_fence,
      adapter: data.adapter,
      set_layout: data.set_layout,
      viewport: data.viewport,
      vertex_buffer: data.vertex_buffer,
      buffer_memory: data.buffer_memory,
      index_buffer: data.index_buffer,
      index_buffer_memory: data.index_buffer_memory,
      desc_set: data.desc_set,
      desc_pool: data.desc_pool,
      queue_group: data.queue_group,
      image_upload_buffer: data.image_upload_buffer,
      image_logo: data.image_logo,
      image_srv: data.image_srv,
      sampler: data.sampler,
      image_memory: data.image_memory,
      image_upload_memory: data.image_upload_memory,
    }
  }

  #[cfg(
    any(
      feature = "vulkan",
      feature = "dx12",
      feature = "metal",
      feature = "gl"
    )
  )]
  pub fn swapchain_stuff(
    surface: &mut <back::Backend as hal::Backend>::Surface,
    device: &mut back::Device,
    physical_device: &back::PhysicalDevice,
    set_layout: &<back::Backend as hal::Backend>::DescriptorSetLayout,
    window: &WindowType,
  ) -> (
    <back::Backend as hal::Backend>::Swapchain,
    <back::Backend as hal::Backend>::RenderPass,
    std::vec::Vec<<back::Backend as hal::Backend>::Framebuffer>,
    std::vec::Vec<(
      <back::Backend as hal::Backend>::Image,
      <back::Backend as hal::Backend>::ImageView,
    )>,
    <back::Backend as hal::Backend>::GraphicsPipeline,
    <back::Backend as hal::Backend>::PipelineLayout,
    Extent2D,
  ) {
    let (caps, formats, _present_modes) = surface.compatibility(physical_device);
    println!("formats: {:?}", formats);
    let format = formats.map_or(f::Format::Rgba8Srgb, |formats| {
      formats
        .iter()
        .find(|format| format.base_format().1 == ChannelType::Srgb)
        .map(|format| *format)
        .unwrap_or(formats[0])
    });

    let extent = match caps.current_extent {
      Some(e) => e,
      None => {
        #[cfg(feature = "gl")]
        let _window = window;
        #[cfg(feature = "gl")]
        let window = surface.get_window();

        let window_size = window
          .get_inner_size()
          .unwrap()
          .to_physical(window.get_hidpi_factor());
        let mut extent = hal::window::Extent2D {
          width: window_size.width as _,
          height: window_size.height as _,
        };

        extent.width = extent
          .width
          .max(caps.extents.start.width)
          .min(caps.extents.end.width);
        extent.height = extent
          .height
          .max(caps.extents.start.height)
          .min(caps.extents.end.height);

        extent
      }
    };

    println!("Surface format: {:?}", format);
    let swap_config = SwapchainConfig::new()
      .with_color(format)
      .with_image_count(caps.image_count.start)
      .with_image_usage(i::Usage::COLOR_ATTACHMENT);
    let (swap_chain, backbuffer) = device.create_swapchain(surface, swap_config, None, &extent);

    let render_pass = {
      let attachment = pass::Attachment {
        format: Some(format),
        samples: 1,
        ops: pass::AttachmentOps::new(
          pass::AttachmentLoadOp::Clear,
          pass::AttachmentStoreOp::Store,
        ),
        stencil_ops: pass::AttachmentOps::DONT_CARE,
        layouts: i::Layout::Undefined..i::Layout::Present,
      };

      let subpass = pass::SubpassDesc {
        colors: &[(0, i::Layout::ColorAttachmentOptimal)],
        depth_stencil: None,
        inputs: &[],
        resolves: &[],
        preserves: &[],
      };

      let dependency = pass::SubpassDependency {
        passes: pass::SubpassRef::External..pass::SubpassRef::Pass(0),
        stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        accesses: i::Access::empty()
          ..(i::Access::COLOR_ATTACHMENT_READ | i::Access::COLOR_ATTACHMENT_WRITE),
      };

      device.create_render_pass(&[attachment], &[subpass], &[dependency])
    };
    let (frame_images, framebuffers) = match backbuffer {
      Backbuffer::Images(images) => {
        let extent = i::Extent {
          width: extent.width as _,
          height: extent.height as _,
          depth: 1,
        };
        let pairs = images
          .into_iter()
          .map(|image| {
            let rtv = device
              .create_image_view(
                &image,
                i::ViewKind::D2,
                format,
                Swizzle::NO,
                COLOR_RANGE.clone(),
              )
              .unwrap();
            (image, rtv)
          })
          .collect::<Vec<_>>();
        let fbos = pairs
          .iter()
          .map(|&(_, ref rtv)| {
            device
              .create_framebuffer(&render_pass, Some(rtv), extent)
              .unwrap()
          })
          .collect();
        (pairs, fbos)
      }
      Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
    };

    let pipeline_layout =
      device.create_pipeline_layout(Some(set_layout), &[(pso::ShaderStageFlags::VERTEX, 0..8)]);
    let pipeline = {
      let vs_module = {
        let glsl = fs::read_to_string("data/quad.vert").unwrap();
        // let glsl = fs::read_to_string("../gfx/examples/quad/data/quad.vert").unwrap();
        let spirv: Vec<u8> = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Vertex)
          .unwrap()
          .bytes()
          .map(|b| b.unwrap())
          .collect();
        device.create_shader_module(&spirv).unwrap()
      };
      let fs_module = {
        let glsl = fs::read_to_string("data/quad.frag").unwrap();
        // let glsl = fs::read_to_string("../gfx/examples/quad/data/quad.frag").unwrap();
        let spirv: Vec<u8> = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Fragment)
          .unwrap()
          .bytes()
          .map(|b| b.unwrap())
          .collect();
        device.create_shader_module(&spirv).unwrap()
      };

      let pipeline = {
        let (vs_entry, fs_entry) = (
          pso::EntryPoint::<back::Backend> {
            entry: ENTRY_NAME,
            module: &vs_module,
            specialization: &[Specialization {
              id: 0,
              value: pso::Constant::F32(0.8),
            }],
          },
          pso::EntryPoint::<back::Backend> {
            entry: ENTRY_NAME,
            module: &fs_module,
            specialization: &[],
          },
        );

        let shader_entries = pso::GraphicsShaderSet {
          vertex: vs_entry,
          hull: None,
          domain: None,
          geometry: None,
          fragment: Some(fs_entry),
        };

        let subpass = Subpass {
          index: 0,
          main_pass: &render_pass,
        };

        let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
          shader_entries,
          Primitive::TriangleList,
          pso::Rasterizer::FILL,
          &pipeline_layout,
          subpass,
        );
        pipeline_desc.blender.targets.push(pso::ColorBlendDesc(
          pso::ColorMask::ALL,
          pso::BlendState::ALPHA,
        ));
        pipeline_desc.vertex_buffers.push(pso::VertexBufferDesc {
          binding: 0,
          stride: std::mem::size_of::<Vertex>() as u32,
          rate: 0,
        });

        pipeline_desc.attributes.push(pso::AttributeDesc {
          location: 0,
          binding: 0,
          element: pso::Element {
            format: f::Format::Rg32Float,
            offset: 0,
          },
        });
        pipeline_desc.attributes.push(pso::AttributeDesc {
          location: 1,
          binding: 0,
          element: pso::Element {
            format: f::Format::Rg32Float,
            offset: 8,
          },
        });

        device.create_graphics_pipeline(&pipeline_desc)
      };

      device.destroy_shader_module(vs_module);
      device.destroy_shader_module(fs_module);

      pipeline.unwrap()
    };

    (
      swap_chain,
      render_pass,
      framebuffers,
      frame_images,
      pipeline,
      pipeline_layout,
      extent,
    )
  }

  pub fn cleanup(data: _WinitWindowData) {
    data
      .device
      .destroy_command_pool(data.command_pool.into_raw());
    data.device.destroy_descriptor_pool(data.desc_pool);
    data.device.destroy_descriptor_set_layout(data.set_layout);

    data.device.destroy_buffer(data.vertex_buffer);
    data.device.destroy_buffer(data.index_buffer);
    data.device.destroy_buffer(data.image_upload_buffer);
    data.device.destroy_image(data.image_logo);
    data.device.destroy_image_view(data.image_srv);
    data.device.destroy_sampler(data.sampler);
    data.device.destroy_fence(data.frame_fence);
    data.device.destroy_semaphore(data.frame_semaphore);
    data.device.destroy_render_pass(data.render_pass);
    data.device.free_memory(data.buffer_memory);
    data.device.free_memory(data.index_buffer_memory);
    data.device.free_memory(data.image_memory);
    data.device.free_memory(data.image_upload_memory);
    data.device.destroy_graphics_pipeline(data.pipeline);
    data.device.destroy_pipeline_layout(data.pipeline_layout);
    for framebuffer in data.framebuffers {
      data.device.destroy_framebuffer(framebuffer);
    }
    for (_, rtv) in data.frame_images {
      data.device.destroy_image_view(rtv);
    }

    data.device.destroy_swapchain(data.swap_chain);
  }
}
