use back;
use back::Backend;
use hal;
use hal::pso::Viewport;
use hal::window::Extent2D;
use hal::{Adapter, CommandPool, Graphics, QueueGroup};

pub struct _WinitWindowData {
  pub device: back::Device,
  pub command_pool: CommandPool<Backend, Graphics>,
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
  pub desc_set: <Backend as hal::Backend>::DescriptorSet,
  pub desc_pool: <Backend as hal::Backend>::DescriptorPool,
  pub queue_group: QueueGroup<Backend, Graphics>,
}
