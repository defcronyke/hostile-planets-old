use color_range::COLOR_RANGE;
use device_state::DeviceState;
use hal;
use hal::format::Swizzle;
use hal::window::Backbuffer;
use hal::{image as i, pool, Backend, Device};
use render_pass_state::RenderPassState;
use std::cell::RefCell;
use std::rc::Rc;
use surface_trait::SurfaceTrait;
use swapchain_state::SwapchainState;

pub struct FramebufferState<B: Backend>
where
  B::Surface: SurfaceTrait,
{
  pub framebuffers: Option<Vec<B::Framebuffer>>,
  pub framebuffer_fences: Option<Vec<B::Fence>>,
  pub command_pools: Option<Vec<hal::CommandPool<B, hal::Graphics>>>,
  pub frame_images: Option<Vec<(B::Image, B::ImageView)>>,
  pub acquire_semaphores: Option<Vec<B::Semaphore>>,
  pub present_semaphores: Option<Vec<B::Semaphore>>,
  pub last_ref: usize,
  pub device: Rc<RefCell<DeviceState<B>>>,
}

impl<B: Backend> FramebufferState<B>
where
  B::Surface: SurfaceTrait,
{
  pub fn new(
    device: Rc<RefCell<DeviceState<B>>>,
    render_pass: &RenderPassState<B>,
    swapchain: &mut SwapchainState<B>,
  ) -> Self {
    let (frame_images, framebuffers) = match swapchain.backbuffer.take().unwrap() {
      Backbuffer::Images(images) => {
        let extent = i::Extent {
          width: swapchain.extent.width as _,
          height: swapchain.extent.height as _,
          depth: 1,
        };
        let pairs = images
          .into_iter()
          .map(|image| {
            let rtv = device
              .borrow()
              .device
              .create_image_view(
                &image,
                i::ViewKind::D2,
                swapchain.format,
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
              .borrow()
              .device
              .create_framebuffer(render_pass.render_pass.as_ref().unwrap(), Some(rtv), extent)
              .unwrap()
          })
          .collect();
        (pairs, fbos)
      }
      Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
    };

    let iter_count = if frame_images.len() != 0 {
      frame_images.len()
    } else {
      1 // GL can have zero
    };

    let mut fences: Vec<B::Fence> = vec![];
    let mut command_pools: Vec<hal::CommandPool<B, hal::Graphics>> = vec![];
    let mut acquire_semaphores: Vec<B::Semaphore> = vec![];
    let mut present_semaphores: Vec<B::Semaphore> = vec![];

    for _ in 0..iter_count {
      fences.push(device.borrow().device.create_fence(true));
      command_pools.push(device.borrow().device.create_command_pool_typed(
        &device.borrow().queues,
        pool::CommandPoolCreateFlags::empty(),
        16,
      ));

      acquire_semaphores.push(device.borrow().device.create_semaphore());
      present_semaphores.push(device.borrow().device.create_semaphore());
    }

    FramebufferState {
      frame_images: Some(frame_images),
      framebuffers: Some(framebuffers),
      framebuffer_fences: Some(fences),
      command_pools: Some(command_pools),
      present_semaphores: Some(present_semaphores),
      acquire_semaphores: Some(acquire_semaphores),
      device,
      last_ref: 0,
    }
  }

  pub fn next_acq_pre_pair_index(&mut self) -> usize {
    if self.last_ref >= self.acquire_semaphores.as_ref().unwrap().len() {
      self.last_ref = 0
    }

    let ret = self.last_ref;
    self.last_ref += 1;
    ret
  }

  pub fn get_frame_data(
    &mut self,
    frame_id: Option<usize>,
    sem_index: Option<usize>,
  ) -> (
    Option<(
      &mut B::Fence,
      &mut B::Framebuffer,
      &mut hal::CommandPool<B, ::hal::Graphics>,
    )>,
    Option<(&mut B::Semaphore, &mut B::Semaphore)>,
  ) {
    (
      if let Some(fid) = frame_id {
        Some((
          &mut self.framebuffer_fences.as_mut().unwrap()[fid],
          &mut self.framebuffers.as_mut().unwrap()[fid],
          &mut self.command_pools.as_mut().unwrap()[fid],
        ))
      } else {
        None
      },
      if let Some(sid) = sem_index {
        Some((
          &mut self.acquire_semaphores.as_mut().unwrap()[sid],
          &mut self.present_semaphores.as_mut().unwrap()[sid],
        ))
      } else {
        None
      },
    )
  }
}

impl<B: Backend> Drop for FramebufferState<B>
where
  B::Surface: SurfaceTrait,
{
  fn drop(&mut self) {
    let device = &self.device.borrow().device;

    for fence in self.framebuffer_fences.take().unwrap() {
      device.wait_for_fence(&fence, !0);
      device.destroy_fence(fence);
    }

    for command_pool in self.command_pools.take().unwrap() {
      device.destroy_command_pool(command_pool.into_raw());
    }

    for acquire_semaphore in self.acquire_semaphores.take().unwrap() {
      device.destroy_semaphore(acquire_semaphore);
    }

    for present_semaphore in self.present_semaphores.take().unwrap() {
      device.destroy_semaphore(present_semaphore);
    }

    for framebuffer in self.framebuffers.take().unwrap() {
      device.destroy_framebuffer(framebuffer);
    }

    for (_, rtv) in self.frame_images.take().unwrap() {
      device.destroy_image_view(rtv);
    }
  }
}
