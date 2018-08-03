use device_state::DeviceState;
use hal::pso::PipelineStage;
use hal::{image as i, pass, Backend, Device};
use std::cell::RefCell;
use std::rc::Rc;
use surface_trait::SurfaceTrait;
use swapchain_state::SwapchainState;

pub struct RenderPassState<B: Backend>
where
  B::Surface: SurfaceTrait,
{
  pub render_pass: Option<B::RenderPass>,
  pub device: Rc<RefCell<DeviceState<B>>>,
}

impl<B: Backend> RenderPassState<B>
where
  B::Surface: SurfaceTrait,
{
  pub fn new(swapchain: &SwapchainState<B>, device: Rc<RefCell<DeviceState<B>>>) -> Self {
    let render_pass = {
      let attachment = pass::Attachment {
        format: Some(swapchain.format.clone()),
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

      device
        .borrow()
        .device
        .create_render_pass(&[attachment], &[subpass], &[dependency])
    };

    RenderPassState {
      render_pass: Some(render_pass),
      device,
    }
  }
}

impl<B: Backend> Drop for RenderPassState<B>
where
  B::Surface: SurfaceTrait,
{
  fn drop(&mut self) {
    let device = &self.device.borrow().device;
    device.destroy_render_pass(self.render_pass.take().unwrap());
  }
}
