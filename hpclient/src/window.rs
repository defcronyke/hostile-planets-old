use back;
use cube::Cube;
use cube_data::_CubeData;
use glsl_to_spirv;
use hal;
use hal::format::{ChannelType, Swizzle};
use hal::pass::Subpass;
use hal::pso::{PipelineStage, ShaderStageFlags, Specialization};
use hal::window::Extent2D;
use hal::{
  adapter, format as f, image as i, pass, pool, pso, Backbuffer, DescriptorPool, Device, Instance,
  PhysicalDevice, Primitive, Surface, SwapchainConfig,
};
use std;
use std::fs;
use std::io::Read;
use vertex::Vertex;
use window_data::_WinitWindowData;
use winit;

#[cfg(feature = "gl")]
use hal::format::{AsFormat, Rgba8Srgb as ColorFormat};

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

  pub fn init(&mut self) -> (_WinitWindowData, Cube, _CubeData) {
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
    let cube_data = cube.init(
      &device,
      &memory_types,
      &limits,
      &desc_set,
      &mut command_pool,
      &mut queue_group,
    );

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

    (
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
        adapter,
        set_layout,
        viewport,
        desc_set,
        desc_pool,
        queue_group,
      },
      cube,
      cube_data,
    )
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
      adapter: data.adapter,
      set_layout: data.set_layout,
      viewport: data.viewport,
      desc_set: data.desc_set,
      desc_pool: data.desc_pool,
      queue_group: data.queue_group,
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
        let spirv: Vec<u8> = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Vertex)
          .unwrap()
          .bytes()
          .map(|b| b.unwrap())
          .collect();
        device.create_shader_module(&spirv).unwrap()
      };
      let fs_module = {
        let glsl = fs::read_to_string("data/quad.frag").unwrap();
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
    data.device.destroy_render_pass(data.render_pass);
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
