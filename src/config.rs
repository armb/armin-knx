use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::hash::Hash;
use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};
use std::thread::Builder;


#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct EibAddr(pub u8, pub u8, pub u8);

impl EibAddr {
    pub fn to_string(&self) -> String {
        format!("{}/{}/{}", self.0, self.1, self.2).to_string()
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub http_listen: Option<String>,
    pub http: Option<HttpConfig>,
    pub knx_server: Option<String>,
    pub knx_multicast_group: Ipv4Addr,
    pub knx_multicast_interface: Ipv4Addr,
    pub rooms: HashMap<String, Room>,
    pub sensors: HashMap<String, Sensor>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfig {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Room {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Actor {
    name: String,
    room: String,
    parameter: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sensor {
    pub name: String,
    pub room: String,
    pub eibaddr: Option<EibAddr>,
}

impl Config {
    pub fn serialize(&self) -> Result<String, ()> {
        serde_json::to_string(self).map_err(|x| { () })
    }
}

// #[derive(Default)]
pub(crate) struct ConfigBuilder {
    http_listen: Option<String>,
    knx_server: Option<String>,
    knx_multicast_group: Ipv4Addr,
    knx_multicast_interface: Ipv4Addr,
    rooms: HashMap<String,Room>,
    sensors: HashMap<String, Sensor>,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder {
            http_listen: None,
            knx_server: None,
            knx_multicast_group: Ipv4Addr::new(224,0,23,12),
            knx_multicast_interface: Ipv4Addr::new(192,168,0,209),
            rooms: HashMap::new(),
            sensors: HashMap::new(),
        }
    }
    pub fn build(self) -> Result<Config, ()> {
        Ok(Config {
            http: None,
            http_listen: Some("0.0.0.0:8080".to_string()),
            knx_server: self.knx_server,
            knx_multicast_group: self.knx_multicast_group,
            knx_multicast_interface: self.knx_multicast_interface,
            rooms: self.rooms,
            sensors: self.sensors,
        })
    }
    pub fn read(mut self, path: &str) -> Result<ConfigBuilder, String> {

        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

        let v: Config = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        self.knx_multicast_interface = v.knx_multicast_interface.clone();
        self.knx_server = v.knx_server.clone();
        self.knx_multicast_group = v.knx_multicast_group.clone();

        // change default values stored in configbuilder if they are available in config-file
        println!("from json:\n{:?}", v);
        for (id, room) in v.rooms {
            println!("- name: {}", room.name);
            self.rooms.insert(id, room);
        }
        for (id, sensor) in v.sensors {
            println!("- name: {}", sensor.name);
            self.sensors.insert(id, sensor);
        }

        println!("configbuilder-rooms: {:?}", self.rooms);
        // panic!("not implemented");
        Ok(self)
    }
}

//
// #[derive(Serialize,Deserialize,Debug)]
// pub struct ConfigFile {
//     pub title: String,
//     pub version: String,
//     pub http: Http,
//     pub rooms: HashMap<String,Room>,
//     pub items: HashMap<String,Item>
// }
//
//
// impl From<serde_json::Error> for Error {
//     fn from(err: serde_json::Error) -> Error {
//         Error::ParseError(err)
//     }
// }
// impl From<std::io::Error> for Error {
//     fn from(err: std::io::Error) -> Error {
//         Error::IoError(err)
//     }
// }
//
//
// #[derive(PartialEq,Serialize,Deserialize,Debug,Clone)]
// pub enum KnxType {
//     Switch,
//     Dimmer,
//     Bell,
//     DoorOpener,
//     Blinds
// }
//
//
// #[derive(Serialize,Deserialize,Debug)]
// pub struct Http {
//     pub listen_port: u16
// }
//
//
// #[derive(Serialize,Deserialize,Debug)]
// pub struct Room {
//     pub name: String
// }
//
// #[derive(Serialize,Deserialize,Debug,Clone)]
// pub struct Item {
//     pub name: String,
//     pub room: String,
//     pub icon: Option<String>,
//     pub knx_type: KnxType,
//     pub knx_write_group: Option<String>,
//     pub knx_read_group: Option<String>,
// }
//
//
// #[derive(Debug)]
// pub struct Config {
//     pub file: ConfigFile,
//     pub base_dir: String
// }
//
//
// pub fn read_from_file(path: &String) -> Result<Arc<Config>,Error> {
//
//     let config_file_data = std::fs::read_to_string(path.to_string() + "res/config.json")?;
//
//     let c: ConfigFile = serde_json::from_str(&config_file_data)?;
//
//     Ok (Arc::new(Config { file: c, base_dir: path.to_string() } ) )
// }
//
//
// impl Config {
//     pub fn get_write_addr(&self, id: &str) -> Result<EibAddr,Error> {
//         match self.file.items.get(id) {
//             // not in database
//             None => Err(Error::NotFound),
//
//             // found item
//             Some(item) => {
//                 match &item.knx_write_group {
//                     // item has no address
//                     None => Err(Error::NotFound),
//                     Some(addr_string) => parse_addr(&addr_string)
//                 }
//             }
//         }
//     }
// }
