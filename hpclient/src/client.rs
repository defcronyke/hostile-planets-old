use conf::ClientConf;
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
use env_logger;
use window_state::WindowState;
use renderer_state::RendererState;
use backend::create_backend;

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
    let msg = "You need to enable the native API feature (vulkan/dx12/metal/gl) in order for this program to work.";
    println!("{}", msg);
    Err(io::Error::from(io::ErrorKind::Other))
  }

  #[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal", feature = "gl"))]
  pub fn run(&self) -> io::Result<()> {
    env_logger::init();

    let mut window = WindowState::new();
    let (backend, _instance) = create_backend(&mut window);

    let mut renderer_state = RendererState::new(backend, window);
    renderer_state.mainloop();

    Ok(())
  }
}
