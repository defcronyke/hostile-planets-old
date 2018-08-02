use conf::ClientConf;
use window::_WinitWindow;
use object::Object;
use asset_loader;
use cpython::PyResult;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::{thread, time};
use toml;
use std::sync::{Arc, RwLock};
use hal::{FrameSync, Swapchain, MemoryType, Device, buffer, memory as m, PhysicalDevice};
use winit;
use env_logger;
use hal;
use cube::Cube;
use cgmath::{Matrix4, Point3, Vector3, perspective, Deg};
use std;
use back;

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
  def load_gltf(&self, path: &str) -> PyResult<i32> {
    let client = self.client(_py);
    let mut objects = client.objects.write().unwrap();
    objects.push(Box::new(asset_loader::load_gltf(path).unwrap()));

    Ok((objects.len() - 1) as i32)
  }
});

pub struct HostilePlanetsClient {
  pub name: String,
  pub conf: ClientConf,
  pub server_con: Option<TcpStream>,
  pub objects: Arc<RwLock<Vec<Box<Object>>>>,
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
    let window_width = 800;
    let window_height = 600;

    let mut w = _WinitWindow::new("cube", window_width, window_height, &events_loop);
    let (mut data, cube, mut cube_data) = w.init();

    // // Create the view matrix for sending to GLSL.
    // let view_matrix = Matrix4::look_at(Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));

    // // Create the projection matrix for sending to GLSL.
    // let near = 0.1;
    // let far = 100.0;
    // let fov = Deg(45.0);
    // let projection_matrix = perspective(fov, window_width as f64 / window_height as f64, near, far);

    // let memory_types = data.adapter.physical_device.memory_properties().memory_types;

    // let model_uniform_buffer = w.create_uniform_buffers(&cube.model_matrix, &view_matrix, &projection_matrix, &data.device, &memory_types);

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

      data.device.reset_fence(&cube_data.frame_fence);
      data.command_pool.reset();
      let frame: hal::SwapImageIndex = {
        match data.swap_chain.acquire_image(FrameSync::Semaphore(&mut cube_data.frame_semaphore)) {
          Ok(i) => i,
          Err(_) => {
            recreate_swapchain = true;
            continue;
          }
        }
      };

      w.update_uniform_data(&data);

      recreate_swapchain = cube.render(&mut data, &mut cube_data, frame, recreate_swapchain);
    }

    // cleanup!
    Cube::cleanup(&data.device, cube_data);
    _WinitWindow::cleanup(data);
    
    Ok(())
  }
}
