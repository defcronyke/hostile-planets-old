use backend_state::BackendState;
use buffer_state::BufferState;
use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};
use color::Color;
use cube::{Cube, CUBE_INDICES};
use desc_set_layout::DescSetLayout;
use device_state::DeviceState;
use dims::DIMS;
use framebuffer_state::FramebufferState;
use hal;
use hal::buffer::IndexBufferView;
use hal::pso::{PipelineStage, ShaderStageFlags};
use hal::queue::submission::Submission;
use hal::window::FrameSync;
use hal::{buffer, command, pool, pso, Backend, Device, IndexType, Swapchain};
use image;
use image_state::ImageState;
use pipeline_state::PipelineState;
use render_pass_state::RenderPassState;
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use surface_trait::SurfaceTrait;
use swapchain_state::SwapchainState;
use uniform::Uniform;
use uniform_matrices_data::UniformMatricesData;
use vertex::Vertex;
use window_state::WindowState;
use winit;

#[cfg(feature = "gl")]
use back::glutin::GlContext;

pub struct RendererState<B: Backend>
where
  B::Surface: SurfaceTrait,
{
  pub uniform_desc_pool: Option<B::DescriptorPool>,
  pub uniform_matrices_desc_pool: Option<B::DescriptorPool>,
  pub img_desc_pool: Option<B::DescriptorPool>,
  pub swapchain: Option<SwapchainState<B>>,
  pub device: Rc<RefCell<DeviceState<B>>>,
  pub backend: BackendState<B>,
  pub window: WindowState,
  pub vertex_buffer: BufferState<B>,
  pub index_buffer: BufferState<B>,
  pub render_pass: RenderPassState<B>,
  pub uniform: Uniform<B>,
  pub uniform_matrices: Uniform<B>,
  pub pipeline: PipelineState<B>,
  pub framebuffer: FramebufferState<B>,
  pub viewport: pso::Viewport,
  pub image: ImageState<B>,
}

impl<B: Backend> RendererState<B>
where
  B::Surface: SurfaceTrait,
{
  pub fn new(mut backend: BackendState<B>, window: WindowState) -> Self {
    let device = Rc::new(RefCell::new(DeviceState::new(
      backend.adapter.adapter.take().unwrap(),
      &backend.surface,
    )));

    let image_desc = DescSetLayout::new(
      Rc::clone(&device),
      vec![
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
    );

    let uniform_desc = DescSetLayout::new(
      Rc::clone(&device),
      vec![pso::DescriptorSetLayoutBinding {
        binding: 0,
        ty: pso::DescriptorType::UniformBuffer,
        count: 1,
        stage_flags: ShaderStageFlags::FRAGMENT,
        immutable_samplers: false,
      }],
    );

    let mut img_desc_pool = Some(device.borrow().device.create_descriptor_pool(
      1, // # of sets
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
    ));

    let mut uniform_desc_pool = Some(device.borrow().device.create_descriptor_pool(
      1, // # of sets
      &[pso::DescriptorRangeDesc {
        ty: pso::DescriptorType::UniformBuffer,
        count: 1,
      }],
    ));

    let image_desc = image_desc.create_desc_set(img_desc_pool.as_mut().unwrap());
    let uniform_desc_built = uniform_desc.create_desc_set(uniform_desc_pool.as_mut().unwrap());

    println!("Memory types: {:?}", backend.adapter.memory_types);

    const IMAGE_LOGO: &'static [u8] = include_bytes!("../../data/images/logo-with-blue-bg.png");
    let img = image::load(Cursor::new(&IMAGE_LOGO[..]), image::PNG)
      .unwrap()
      .to_rgba();

    let mut staging_pool = device.borrow().device.create_command_pool_typed(
      &device.borrow().queues,
      pool::CommandPoolCreateFlags::empty(),
      16,
    );

    let image = ImageState::new::<hal::Graphics>(
      image_desc,
      &img,
      &backend.adapter,
      buffer::Usage::TRANSFER_SRC,
      &mut device.borrow_mut(),
      &mut staging_pool,
    );

    let cube = Cube::new();

    let vertex_buffer = BufferState::new::<Vertex>(
      Rc::clone(&device),
      &cube.vertices,
      buffer::Usage::VERTEX,
      &backend.adapter.memory_types,
    );

    let index_buffer = BufferState::new::<u16>(
      Rc::clone(&device),
      &cube.indices,
      buffer::Usage::INDEX,
      &backend.adapter.memory_types,
    );

    let uniform = Uniform::new(
      Rc::clone(&device),
      &backend.adapter.memory_types,
      &[1.0f32, 1.0f32, 1.0f32, 1.0f32],
      uniform_desc_built,
      0,
    );

    // Model matrix
    let model = cube.model_matrix;

    // View matrix
    let view: Matrix4<f32> = Matrix4::look_at(
      Point3::new(5.0, 5.0, 5.0),
      Point3::new(0.0, 0.0, 0.0),
      Vector3::new(0.0, 1.0, 0.0),
    );

    // Projection matrix
    let fovy = Deg(45.0);
    let aspect = DIMS.width as f32 / DIMS.height as f32;
    let near = 0.1;
    let far = 100.0;
    let proj = perspective(fovy, aspect, near, far);

    // Vulkan clip matrix
    let clip: Matrix4<f32> = Matrix4::new(
      1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
    );

    let proj = clip * proj;

    let uniform_matrices_data = UniformMatricesData::new(model, view, proj);

    let uniform_matrices_desc = DescSetLayout::new(
      Rc::clone(&device),
      vec![pso::DescriptorSetLayoutBinding {
        binding: 0,
        ty: pso::DescriptorType::UniformBuffer,
        count: 1,
        stage_flags: ShaderStageFlags::VERTEX,
        immutable_samplers: false,
      }],
    );

    let mut uniform_matrices_desc_pool = Some(device.borrow().device.create_descriptor_pool(
      1, // # of sets
      &[pso::DescriptorRangeDesc {
        ty: pso::DescriptorType::UniformBuffer,
        count: 1,
      }],
    ));

    let uniform_matrices_desc_built =
      uniform_matrices_desc.create_desc_set(uniform_matrices_desc_pool.as_mut().unwrap());

    let uniform_matrices = Uniform::new(
      Rc::clone(&device),
      &backend.adapter.memory_types,
      &[uniform_matrices_data],
      uniform_matrices_desc_built,
      0,
    );

    image.wait_for_transfer_completion();

    device
      .borrow()
      .device
      .destroy_command_pool(staging_pool.into_raw());

    let mut swapchain = Some(SwapchainState::new(&mut backend, Rc::clone(&device)));

    let render_pass = RenderPassState::new(swapchain.as_ref().unwrap(), Rc::clone(&device));

    let framebuffer = FramebufferState::new(
      Rc::clone(&device),
      &render_pass,
      swapchain.as_mut().unwrap(),
    );

    let pipeline = PipelineState::new(
      vec![
        image.get_layout(),
        uniform.get_layout(),
        uniform_matrices.get_layout(),
      ],
      render_pass.render_pass.as_ref().unwrap(),
      Rc::clone(&device),
    );

    let viewport = RendererState::create_viewport(swapchain.as_ref().unwrap());

    RendererState {
      window,
      backend,
      device,
      image,
      img_desc_pool,
      uniform_desc_pool,
      uniform_matrices_desc_pool,
      vertex_buffer,
      index_buffer,
      uniform,
      uniform_matrices,
      render_pass,
      pipeline,
      swapchain,
      framebuffer,
      viewport,
    }
  }

  pub fn recreate_swapchain(&mut self) {
    self.device.borrow().device.wait_idle().unwrap();

    self.swapchain.take().unwrap();

    self.swapchain = Some(SwapchainState::new(
      &mut self.backend,
      Rc::clone(&self.device),
    ));

    self.render_pass =
      RenderPassState::new(self.swapchain.as_ref().unwrap(), Rc::clone(&self.device));

    self.framebuffer = FramebufferState::new(
      Rc::clone(&self.device),
      &self.render_pass,
      self.swapchain.as_mut().unwrap(),
    );

    self.pipeline = PipelineState::new(
      vec![
        self.image.get_layout(),
        self.uniform.get_layout(),
        self.uniform_matrices.get_layout(),
      ],
      self.render_pass.render_pass.as_ref().unwrap(),
      Rc::clone(&self.device),
    );

    self.viewport = RendererState::create_viewport(self.swapchain.as_ref().unwrap());
  }

  pub fn create_viewport(swapchain: &SwapchainState<B>) -> pso::Viewport {
    pso::Viewport {
      rect: pso::Rect {
        x: 0,
        y: 0,
        w: swapchain.extent.width as i16,
        h: swapchain.extent.height as i16,
      },
      depth: 0.0..1.0,
    }
  }

  pub fn mainloop(&mut self) {
    let mut running = true;
    let mut recreate_swapchain = false;

    let mut r = 1.0f32;
    let mut g = 1.0f32;
    let mut b = 1.0f32;
    let mut a = 1.0f32;

    let mut cr = 0.8;
    let mut cg = 0.8;
    let mut cb = 0.8;

    let mut cur_color = Color::Red;
    let mut cur_value: u32 = 0;

    println!("\nInstructions:");
    println!("\tChoose whether to change the (R)ed, (G)reen or (B)lue color by pressing the appropriate key.");
    println!("\tType in the value you want to change it to, where 0 is nothing, 255 is normal and 510 is double, ect.");
    println!("\tThen press C to change the (C)lear colour or (Enter) for the image color.");
    println!(
      "\tSet {:?} color to: {} (press enter/C to confirm)",
      cur_color, cur_value
    );

    while running {
      {
        let uniform = &mut self.uniform;
        // let uniform_matrices = &mut self.uniform_matrices;
        #[cfg(feature = "gl")]
        let backend = &self.backend;

        self.window.events_loop.poll_events(|event| {
          if let winit::Event::WindowEvent { event, .. } = event {
            #[allow(unused_variables)]
            match event {
              winit::WindowEvent::KeyboardInput {
                input:
                  winit::KeyboardInput {
                    virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                    ..
                  },
                ..
              }
              | winit::WindowEvent::CloseRequested => running = false,
              winit::WindowEvent::Resized(dims) => {
                #[cfg(feature = "gl")]
                backend
                  .surface
                  .get_window_t()
                  .resize(dims.to_physical(backend.surface.get_window_t().get_hidpi_factor()));
                recreate_swapchain = true;
              }
              winit::WindowEvent::KeyboardInput {
                input:
                  winit::KeyboardInput {
                    virtual_keycode,
                    state: winit::ElementState::Pressed,
                    ..
                  },
                ..
              } => {
                if let Some(kc) = virtual_keycode {
                  match kc {
                    winit::VirtualKeyCode::Key0 => cur_value = cur_value * 10 + 0,
                    winit::VirtualKeyCode::Key1 => cur_value = cur_value * 10 + 1,
                    winit::VirtualKeyCode::Key2 => cur_value = cur_value * 10 + 2,
                    winit::VirtualKeyCode::Key3 => cur_value = cur_value * 10 + 3,
                    winit::VirtualKeyCode::Key4 => cur_value = cur_value * 10 + 4,
                    winit::VirtualKeyCode::Key5 => cur_value = cur_value * 10 + 5,
                    winit::VirtualKeyCode::Key6 => cur_value = cur_value * 10 + 6,
                    winit::VirtualKeyCode::Key7 => cur_value = cur_value * 10 + 7,
                    winit::VirtualKeyCode::Key8 => cur_value = cur_value * 10 + 8,
                    winit::VirtualKeyCode::Key9 => cur_value = cur_value * 10 + 9,
                    winit::VirtualKeyCode::R => {
                      cur_value = 0;
                      cur_color = Color::Red
                    }
                    winit::VirtualKeyCode::G => {
                      cur_value = 0;
                      cur_color = Color::Green
                    }
                    winit::VirtualKeyCode::B => {
                      cur_value = 0;
                      cur_color = Color::Blue
                    }
                    winit::VirtualKeyCode::A => {
                      cur_value = 0;
                      cur_color = Color::Alpha
                    }
                    winit::VirtualKeyCode::Return => {
                      match cur_color {
                        Color::Red => r = cur_value as f32 / 255.0,
                        Color::Green => g = cur_value as f32 / 255.0,
                        Color::Blue => b = cur_value as f32 / 255.0,
                        Color::Alpha => a = cur_value as f32 / 255.0,
                      }
                      uniform
                        .buffer
                        .as_mut()
                        .unwrap()
                        .update_data(0, &[r, g, b, a]);
                      cur_value = 0;

                      println!("Colour updated!");
                    }
                    winit::VirtualKeyCode::C => {
                      match cur_color {
                        Color::Red => cr = cur_value as f32 / 255.0,
                        Color::Green => cg = cur_value as f32 / 255.0,
                        Color::Blue => cb = cur_value as f32 / 255.0,
                        Color::Alpha => {
                          error!("Alpha is not valid for the background.");
                          return;
                        }
                      }
                      cur_value = 0;

                      println!("Background color updated!");
                    }
                    _ => return,
                  }
                  println!(
                    "Set {:?} color to: {} (press enter/C to confirm)",
                    cur_color, cur_value
                  )
                }
              }
              _ => (),
            }
          }
        });
      }

      if recreate_swapchain {
        self.recreate_swapchain();
        recreate_swapchain = false;
      }

      let sem_index = self.framebuffer.next_acq_pre_pair_index();

      let frame: hal::SwapImageIndex = {
        let (acquire_semaphore, _) = self
          .framebuffer
          .get_frame_data(None, Some(sem_index))
          .1
          .unwrap();
        match self
          .swapchain
          .as_mut()
          .unwrap()
          .swapchain
          .as_mut()
          .unwrap()
          .acquire_image(FrameSync::Semaphore(acquire_semaphore))
        {
          Ok(i) => i,
          Err(_) => {
            recreate_swapchain = true;
            continue;
          }
        }
      };

      let (fid, sid) = self
        .framebuffer
        .get_frame_data(Some(frame as usize), Some(sem_index));

      let (framebuffer_fence, framebuffer, command_pool) = fid.unwrap();
      let (image_acquired, image_present) = sid.unwrap();

      self
        .device
        .borrow()
        .device
        .wait_for_fence(framebuffer_fence, !0);
      self.device.borrow().device.reset_fence(framebuffer_fence);
      command_pool.reset();

      // Rendering
      let submit = {
        let mut cmd_buffer = command_pool.acquire_command_buffer(false);

        cmd_buffer.set_viewports(0, &[self.viewport.clone()]);
        cmd_buffer.set_scissors(0, &[self.viewport.rect]);
        // cmd_buffer.set_depth_bounds(0.0..100.0);
        cmd_buffer.bind_graphics_pipeline(self.pipeline.pipeline.as_ref().unwrap());
        cmd_buffer.bind_vertex_buffers(0, Some((self.vertex_buffer.get_buffer(), 0)));
        cmd_buffer.bind_index_buffer(IndexBufferView {
          buffer: self.index_buffer.get_buffer(),
          offset: 0,
          index_type: IndexType::U16,
        });
        cmd_buffer.bind_graphics_descriptor_sets(
          self.pipeline.pipeline_layout.as_ref().unwrap(),
          0,
          vec![
            self.image.desc.set.as_ref().unwrap(),
            self.uniform.desc.as_ref().unwrap().set.as_ref().unwrap(),
            self
              .uniform_matrices
              .desc
              .as_ref()
              .unwrap()
              .set
              .as_ref()
              .unwrap(),
          ],
          &[],
        ); //TODO

        {
          let mut encoder = cmd_buffer.begin_render_pass_inline(
            self.render_pass.render_pass.as_ref().unwrap(),
            framebuffer,
            self.viewport.rect,
            &[command::ClearValue::Color(command::ClearColor::Float([
              cr, cg, cb, 1.0,
            ]))],
          );

          encoder.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0..1);
          // encoder.draw(0..6, 0..1);
        }

        cmd_buffer.finish()
      };

      let submission = Submission::new()
        .wait_on(&[(&*image_acquired, PipelineStage::BOTTOM_OF_PIPE)])
        .signal(&[&*image_present])
        .submit(Some(submit));
      self.device.borrow_mut().queues.queues[0].submit(submission, Some(framebuffer_fence));

      // present frame
      if let Err(_) = self
        .swapchain
        .as_ref()
        .unwrap()
        .swapchain
        .as_ref()
        .unwrap()
        .present(
          &mut self.device.borrow_mut().queues.queues[0],
          frame,
          Some(&*image_present),
        ) {
        recreate_swapchain = true;
        continue;
      }
    }
  }
}

impl<B: Backend> Drop for RendererState<B>
where
  B::Surface: SurfaceTrait,
{
  fn drop(&mut self) {
    self.device.borrow().device.wait_idle().unwrap();
    self
      .device
      .borrow()
      .device
      .destroy_descriptor_pool(self.img_desc_pool.take().unwrap());
    self
      .device
      .borrow()
      .device
      .destroy_descriptor_pool(self.uniform_desc_pool.take().unwrap());
    self
      .device
      .borrow()
      .device
      .destroy_descriptor_pool(self.uniform_matrices_desc_pool.take().unwrap());
    self.swapchain.take();
  }
}
