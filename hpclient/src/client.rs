use conf::*;
// use window::*;
// use cube::*;
// use object::*;
use gltf_object::*;
// use asset_loader;

use cpython::PyResult;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::{thread, time};
use toml;
// use piston_window::{RenderEvent, ResizeEvent};
// use camera_controllers::{
//   FirstPerson,
//   FirstPersonSettings
// };
use std::sync::{Arc, RwLock};

py_module_initializer!(hpclient, inithpclient, PyInit_hpclient, |py, m| {
  try!(m.add(py, "__doc__", "This module is implemented in Rust."));
  try!(m.add_class::<Client>(py));
  
  Ok(())
});

py_class!(class Client |py| {
  data client: HostilePlanetsClient;

  def __new__(_cls, conf_path: &str) -> PyResult<Self> {
    let c = HostilePlanetsClient::new(conf_path);

    Self::create_instance(py, c)
  }

  def connect(&self) -> PyResult<i32> {
    let client = self.client(py);
    py.allow_threads(|| {
        client.connect().unwrap();
    });

    Ok(0)
  }

  def connect_to(&self, address: &str) -> PyResult<i32> {
    let client = self.client(py);
    py.allow_threads(|| {
        client.connect_to(address).unwrap();
    });
    
    Ok(0)
  }

  def get_conf(&self) -> PyResult<ClientConf> {
    let client = self.client(py);
    let conf = client.conf.clone();

    Ok(conf)
  }

  def run(&self) -> PyResult<i32> {
    self.client(py).run().unwrap();

    Ok(0)
  }

  // Loads the GLTF file from disk and parses it, then saves it into
  // the objects vector.
  //
  // Returns the index that the object is saved at in the objects vector.
  def load_gltf(&self, _path: &str) -> PyResult<i32> {
    let client = self.client(py);
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

  pub fn run(&self) -> io::Result<()> {
    // let mut pw = _PistonWindow::new();
    // let cube = Cube::new(&mut pw);
    // let w = &mut pw.window;

    // let mut first_person = FirstPerson::new(
    //   [0.5, 0.5, 4.0],
    //   FirstPersonSettings::keyboard_wasd()
    // );

    // let objects = self.objects.write().unwrap();
    
    // for mut obj in objects.clone() {
    //   match obj.init(w, &first_person) {
    //     _ => {
    //       obj.set_projection(w).unwrap();
    //       ()
    //     },
    //   }
    // }
    
    // while let Some(e) = w.next() {
    //   // let mut first_person = &mut first_person;
    //   // first_person.event(&e);

    //   w.draw_3d(&e, |w| {
    //     let args = e.render_args().unwrap();
    //     w.encoder.clear(&w.output_color, [0.3, 0.3, 0.3, 1.0]);
    //     w.encoder.clear_depth(&w.output_stencil, 1.0);
    //     // cube.draw(w, &args, &first_person).unwrap();
    //     let objects = self.objects.write().unwrap();

    //     for mut obj in objects.clone() {
    //       // match obj.draw(w, &args, &first_person) {
    //       //   _ => {
    //       //     obj.set_projection(w).unwrap();
    //       //     ()
    //       //   },
    //       // }
    //     }
    //   });

    //   if let Some(_) = e.resize_args() {
    //     cube.reset(w).unwrap();
    //     let objects = self.objects.write().unwrap();

    //     for mut obj in objects.clone() {
    //       match obj.reset(w) {
    //         _ => (),
    //       }
    //     }
    //   }
    // } 
    
    Ok(())
  }
}
