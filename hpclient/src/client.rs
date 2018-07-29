use conf::ClientConf;
use window::_WinitWindow;
use gltf_object::GltfObject;
use cpython::PyResult;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::{thread, time};
use toml;
use std::sync::{Arc, RwLock};
use hal::pso::PipelineStage;
use hal::queue::Submission;
use hal::{command, FrameSync, Device, Swapchain};
use winit;
use env_logger;
use hal;

#[cfg(feature = "gl")]
use back::glutin::GlContext;


#[cfg(feature = "vulkan")]
py_module_initializer!(hpclient_vulkan, inithpclient_vulkan, PyInit_hpclient_vulkan, |py, m| {
  try!(m.add(py, "__doc__", "This module is implemented in Rust."));
  try!(m.add_class::<Client>(py));
  
  Ok(())
});

#[cfg(feature = "dx12")]
py_module_initializer!(hpclient_dx12, inithpclient_dx12, PyInit_hpclient_dx12, |py, m| {
  try!(m.add(py, "__doc__", "This module is implemented in Rust."));
  try!(m.add_class::<Client>(py));
  
  Ok(())
});

#[cfg(feature = "gl")]
py_module_initializer!(hpclient_gl, inithpclient_gl, PyInit_hpclient_gl, |py, m| {
  try!(m.add(py, "__doc__", "This module is implemented in Rust."));
  try!(m.add_class::<Client>(py));
  
  Ok(())
});

py_class!(class Client |_py| {
  data client: HostilePlanetsClient;

  def __new__(_cls, conf_path: &str) -> PyResult<Self> {
    let c = HostilePlanetsClient::new(conf_path);

    Self::create_instance(_py, c)
  }

  def client_type(&self) -> PyResult<String> {
    Ok(HostilePlanetsClient::client_type())
  }

  def connect(&self) -> PyResult<i32> {
    let client = self.client(_py);
    _py.allow_threads(|| {
        client.connect().unwrap();
    });

    Ok(0)
  }

  def connect_to(&self, address: &str) -> PyResult<i32> {
    let client = self.client(_py);
    _py.allow_threads(|| {
        client.connect_to(address).unwrap();
    });
    
    Ok(0)
  }

  def get_conf(&self) -> PyResult<ClientConf> {
    let client = self.client(_py);
    let conf = client.conf.clone();

    Ok(conf)
  }

  def run(&self) -> PyResult<i32> {
    self.client(_py).run().unwrap();

    Ok(0)
  }

  // Loads the GLTF file from disk and parses it, then saves it into
  // the objects vector.
  //
  // Returns the index that the object is saved at in the objects vector.
  def load_gltf(&self, _path: &str) -> PyResult<i32> {
    let client = self.client(_py);
    let objects = client.objects.write().unwrap();
    // objects.push(asset_loader::load_gltf(path).unwrap());

    Ok((objects.clone().len() - 1) as i32)
  }
});

pub struct HostilePlanetsClient {
  pub name: String,
  pub conf: ClientConf,
  pub server_con: Option<TcpStream>,
  pub objects: Arc<RwLock<Vec<GltfObject>>>,
}

pub trait _Client {
  // Client type.
  #[cfg(feature = "vulkan")]
  fn client_type() -> String {
    String::from("Vulkan")
  }

  #[cfg(feature = "dx12")]
  fn client_type() -> String {
    String::from("DirectX 12")
  }

  #[cfg(feature = "gl")]
  fn client_type() -> String {
    String::from("OpenGL")
  }

  // Connect to a server by address, such as: "127.0.0.1:8080"
  fn connect_to(&self, address: &str) -> io::Result<TcpStream> {
    // Connect to the server.
    println!("connecting to server: {} ...", address);
    match TcpStream::connect(address) {
      Ok(stream) => {
        println!("connected to server: {:?}", stream);

        // Wait for the server to say hello, and print the welcome message.
        let mut welcome_msg = String::new();

        let mut reader = BufReader::new(stream.try_clone().unwrap());
        reader.read_line(&mut welcome_msg).unwrap();

        print!("server msg: {}", welcome_msg);

        Ok(stream)
      },

      Err(e) => {
        println!("failed connecting to server: {:?} : retrying in 10 seconds ...", e);
        thread::sleep(time::Duration::from_millis(1000 * 10));

        self.connect_to(address)
      },
    }
  }
}

impl _Client for HostilePlanetsClient {}

impl HostilePlanetsClient {
  pub fn new(conf_path: &str) -> Self {
    let name = String::from("Hostile Planets client");
    println!("loading {} ...", name);

    let mut f = File::open(conf_path).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
      .expect("something went wrong reading the file");

    let conf: ClientConf = toml::from_str(&contents).unwrap();

    println!("using config {}: {:?}", conf_path, conf);

    let c = Self {
      name: name.clone(),
      conf: conf,
      server_con: None,
      objects: Arc::new(RwLock::new(Vec::new())),
    };

    println!("{} loaded", name);

    c
  }

  // Connect to the server specified in conf.toml.
  pub fn connect(&self) -> io::Result<TcpStream> {
    // Connect to the server.
    let addr = format!("{}:{}", self.conf.client.ip, self.conf.client.port);
    self.connect_to(&addr)
  }

  #[cfg(not(any(feature = "vulkan", feature = "dx12", feature = "metal", feature = "gl")))]
  pub fn run(&self) -> io::Result<()> {
    let msg = "You need to enable the native API feature (vulkan/dx12/metal/gl) in order to test the LL";
    println!("{}", msg);
    Err(io::Error::from(io::ErrorKind::Other))
  }

  #[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal", feature = "gl"))]
  pub fn run(&self) -> io::Result<()> {
    env_logger::init();

    let mut events_loop = winit::EventsLoop::new();
    let mut w = _WinitWindow::new("quad", 800, 600, &events_loop);
    let mut data = w.init();

    let mut running = true;
    let mut recreate_swapchain = false;
    while running {
      events_loop.poll_events(|event| {
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
              w.surface
                .get_window()
                .resize(dims.to_physical(w.surface.get_window().get_hidpi_factor()));
              recreate_swapchain = true;
            }
            _ => (),
          }
        }
      });

      if recreate_swapchain {
        data = _WinitWindow::recreate_swapchain(&w.window, &mut w.surface, data);
        recreate_swapchain = false;
      }

      data.device.reset_fence(&data.frame_fence);
      data.command_pool.reset();
      let frame: hal::SwapImageIndex = {
        match data.swap_chain.acquire_image(FrameSync::Semaphore(&mut data.frame_semaphore)) {
          Ok(i) => i,
          Err(_) => {
            recreate_swapchain = true;
            continue;
          }
        }
      };

      // Rendering
      let submit = {
        let mut cmd_buffer = data.command_pool.acquire_command_buffer(false);

        cmd_buffer.set_viewports(0, &[data.viewport.clone()]);
        cmd_buffer.set_scissors(0, &[data.viewport.rect]);
        cmd_buffer.bind_graphics_pipeline(&data.pipeline);
        cmd_buffer.bind_vertex_buffers(0, Some((&data.vertex_buffer, 0)));
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
          encoder.draw(0..6, 0..1);
        }

        cmd_buffer.finish()
      };

      let submission = Submission::new()
        .wait_on(&[(&data.frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)])
        .submit(Some(submit));
      data.queue_group.queues[0].submit(submission, Some(&mut data.frame_fence));

      // TODO: replace with semaphore
      data.device.wait_for_fence(&data.frame_fence, !0);

      // present frame
      if let Err(_) = data.swap_chain.present(&mut data.queue_group.queues[0], frame, &[]) {
        recreate_swapchain = true;
      }
    }

    // cleanup!
    data.device.destroy_command_pool(data.command_pool.into_raw());
    data.device.destroy_descriptor_pool(data.desc_pool);
    data.device.destroy_descriptor_set_layout(data.set_layout);

    data.device.destroy_buffer(data.vertex_buffer);
    data.device.destroy_buffer(data.image_upload_buffer);
    data.device.destroy_image(data.image_logo);
    data.device.destroy_image_view(data.image_srv);
    data.device.destroy_sampler(data.sampler);
    data.device.destroy_fence(data.frame_fence);
    data.device.destroy_semaphore(data.frame_semaphore);
    data.device.destroy_render_pass(data.render_pass);
    data.device.free_memory(data.buffer_memory);
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
    
    Ok(())
  }
}
