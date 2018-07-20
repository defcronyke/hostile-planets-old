extern crate toml;

use conf::*;
use player::*;
use unit::*;

use cpython::PyResult;
use std::borrow::Borrow;
use std::cmp::Eq;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

py_module_initializer!(hpserver, inithpserver, PyInit_hpserver, |py, m| {
    try!(m.add(py, "__doc__", "This module is implemented in Rust."));
    try!(m.add_class::<Server>(py));
    
    Ok(())
});

py_class!(class Server |py| {
    data server: _Server;

    def __new__(_cls, conf_path: &str) -> PyResult<Self> {
        let s = _Server::new(conf_path);

        Self::create_instance(py, s)
    }

    def get_name(&self) -> PyResult<String> {
        let server = self.server(py);
        let name = server.data.read().unwrap().name.to_string();

        Ok(name)
    }

    def get_conf(&self) -> PyResult<ServerConf> {
        let server = self.server(py);
        let conf = server.data.read().unwrap().conf.clone();

        Ok(conf)
    }

    def get_players(&self) -> PyResult<HashMap<String, _Player>> {
        let server = self.server(py);
        let players = server.data.read().unwrap().players.clone();
        
        Ok(players)
    }

    def is_connected(&self, name: &str) -> PyResult<bool> {
        let server = self.server(py);
        match server.data.read().unwrap().cons.get(name) {
            Some(res) => {
                match res {
                    Some(_) => {
                        println!("player {} is connected", name);
                        Ok(true)
                    },
                    None => {
                        println!("player {} is known but not connected", name);
                        Ok(false)
                    }
                }
            },
            None => {
                println!("player {} is not connected", name);
                Ok(false)
            }
        }
    }

    def listen(&self) -> PyResult<i32> {
        let server = self.server(py);
        py.allow_threads(|| {
            server.listen().unwrap();
        });

        Ok(0)
    }

    def listen_to(&self, address: &str) -> PyResult<i32> {
        let server = self.server(py);
        py.allow_threads(|| {
            server.listen_to(address).unwrap();
        });

        Ok(0)
    }
});

type _Connections = HashMap<String, Option<TcpStream>>;

pub struct _Data {
    name: String,
    conf: ServerConf,
    players: HashMap<String, _Player>,
    cons: _Connections,
}

pub struct _Server {
    data: Arc<RwLock<_Data>>,
}

impl _Server {
    pub fn new(conf_path: &str) -> Self {
        let name = String::from("Hostile Planets server");
        println!("Loading {} ...", name);

        let mut f = File::open(conf_path).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        let conf: ServerConf = toml::from_str(&contents).unwrap();
        println!("using config {}: {:?}", conf_path, conf);

        let player_name = "default player";
        let mut p = HashMap::new();
        p.insert(player_name.to_string(), _Player::new(player_name, Units::new()));

        let mut cons = HashMap::new();
        cons.insert(player_name.to_string(), None);

        _Server {
            data: Arc::new(RwLock::new(_Data {
                name: name,
                conf: conf,
                players: p,
                cons: cons,    
            }))
        }
    }

    pub fn listen(&self) -> io::Result<()> {
        let server_conf = self.data.read().unwrap().conf.clone().server;
        let ip = server_conf.ip;
        let port = server_conf.port;
        let address = format!("{}:{}", ip, port);
        
        self.listen_to(&address).unwrap();

        Ok(())
    }

    pub fn listen_to(&self, address: &str) -> io::Result<()> {
        let listener = TcpListener::bind(address).unwrap();

        {
            let name = self.data.read().unwrap().name.clone();
            println!("{} listening on: {}", name, address);
        }

        for mut stream in listener.incoming() {
            let key = String::from("Henry");

            {
                let mut players = self.data.write().unwrap().players.clone();
                players.insert(
                    key.clone(),
                    _Player::new(&key, HashMap::new()),
                );
            }

            {
                self.data.write().unwrap().cons.insert(key.clone(), Some(stream.unwrap()));
            }
            
            self.handle_client(key);
        }

        Ok(())
    }

    fn handle_client<'a, T>(&self, key: T)
    where
        T: Hash + Eq,
        String: Borrow<T>,
    {
        let welcome_msg = "Welcome to Hostile Planets\n";

        {
            let data = self.data.read().unwrap();
            let players = data.players.clone();
            let player = players.get(&key);

            let con = self.data.read().unwrap().cons.get(&key).unwrap().as_ref().unwrap().try_clone().unwrap();
            println!("client connected: {:?} : {:?}", player, con);
            println!("players: {:?}", players);
        }

        {
            let data = self.data.write().unwrap();
            let mut con = data.cons.get(&key).unwrap().as_ref().unwrap().try_clone().unwrap();
            con.write(welcome_msg.as_bytes()).unwrap();
        }
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
