use back::Backend;
use hal;

pub struct _CubeData {
  pub frame_semaphore: <Backend as hal::Backend>::Semaphore,
  pub frame_fence: <Backend as hal::Backend>::Fence,
  pub vertex_buffer: <Backend as hal::Backend>::Buffer,
  pub buffer_memory: <Backend as hal::Backend>::Memory,
  pub index_buffer: <Backend as hal::Backend>::Buffer,
  pub index_buffer_memory: <Backend as hal::Backend>::Memory,
  pub image_upload_buffer: <Backend as hal::Backend>::Buffer,
  pub image_logo: <Backend as hal::Backend>::Image,
  pub image_srv: <Backend as hal::Backend>::ImageView,
  pub sampler: <Backend as hal::Backend>::Sampler,
  pub image_memory: <Backend as hal::Backend>::Memory,
  pub image_upload_memory: <Backend as hal::Backend>::Memory,
}
