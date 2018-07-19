use cpython::{Python, ToPyObject, PyDict};
use std::clone::Clone;

// #[derive(Deserialize, Debug, Clone)]
// pub struct ServerConf {
//     pub server: ServerConfServer,
// }

// impl ToPyObject for ServerConf {
//     type ObjectType = PyDict;

//     fn to_py_object(&self, py: Python) -> PyDict {
//         let dict = PyDict::new(py);
//         dict.set_item(py, "server", self.server.clone()).unwrap();
//         dict
//     }
// }

// #[derive(Deserialize, Debug, Clone)]
// pub struct ServerConfServer {
//     pub ip: String,
//     pub port: u32,
//     pub maps: Vec<ServerConfMap>,
// }

// impl ToPyObject for ServerConfServer {
//     type ObjectType = PyDict;

//     fn to_py_object(&self, py: Python) -> PyDict {
//         let dict = PyDict::new(py);
//         dict.set_item(py, "ip", self.ip.clone()).unwrap();
//         dict.set_item(py, "port", self.port.clone()).unwrap();
//         dict.set_item(py, "maps", self.maps.clone()).unwrap();
//         dict
//     }
// }

// #[derive(Deserialize, Debug, Clone)]
// pub struct ServerConfMap {
//     pub name: String,
//     pub script: String,
// }

// impl ToPyObject for ServerConfMap {
//     type ObjectType = PyDict;

//     fn to_py_object(&self, py: Python) -> PyDict {
//         let dict = PyDict::new(py);
//         dict.set_item(py, "name", self.name.clone()).unwrap();
//         dict.set_item(py, "script", self.script.clone()).unwrap();
//         dict
//     }
// }

#[derive(Deserialize, Debug, Clone)]
pub struct ClientConf {
    pub client: ClientConfClient,
}

impl ToPyObject for ClientConf {
    type ObjectType = PyDict;

    fn to_py_object(&self, py: Python) -> PyDict {
        let dict = PyDict::new(py);
        dict.set_item(py, "client", self.client.clone()).unwrap();

        dict
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientConfClient {
    pub ip: String,
    pub port: u32,
    pub players: Vec<ClientConfPlayer>,
}

impl ToPyObject for ClientConfClient {
    type ObjectType = PyDict;

    fn to_py_object(&self, py: Python) -> PyDict {
        let dict = PyDict::new(py);
        dict.set_item(py, "ip", self.ip.clone()).unwrap();
        dict.set_item(py, "port", self.port.clone()).unwrap();
        dict.set_item(py, "players", self.players.clone()).unwrap();
        
        dict
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientConfPlayer {
    pub name: String,
}

impl ToPyObject for ClientConfPlayer {
    type ObjectType = PyDict;

    fn to_py_object(&self, py: Python) -> PyDict {
        let dict = PyDict::new(py);
        dict.set_item(py, "name", self.name.clone()).unwrap();
        
        dict
    }
}
