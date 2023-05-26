use std::collections::{HashMap};
use std::error::Error;
use std::hash::Hash;
use std::net::{Ipv4Addr, SocketAddrV4};
use serde::{Deserialize, Serialize};
use crate::data::Dimension;


#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Hash, Eq)]
pub struct EibAddr(pub u8, pub u8, pub u8);

impl EibAddr {
    pub fn to_string(&self) -> String {
        format!("{}/{}/{}", self.0, self.1, self.2).to_string()
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Room {
    // pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Actor {
    pub name: String,
    pub room_id: String,
    pub commands: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sensor {
    pub dimension: String,
    pub name: String,
    pub room_id: String,
    pub eibaddr: String,
}

impl Sensor {
    pub fn get_dimension(&self) -> Dimension {
        match self.dimension.as_str() {
            "brightness" => Dimension::Brightness,
            "temperature" => Dimension::Temperature,
            "onoff" => Dimension::OnOff,
            _ => Dimension::None
        }
    }
}





#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub http_listen_address: String,
    pub knx_server: Option<String>,
    pub knx_multicast_group: Ipv4Addr,
    pub knx_multicast_interface: Ipv4Addr,
    pub knx_multicast_port: u16,
    pub room_list: Vec<String>,
    pub rooms: HashMap<String, Room>,
    pub sensors: HashMap<String, Sensor>,
    pub actors: HashMap<String, Actor>,
}


impl Config {
    pub fn serialize(&self) -> Result<String, ()> {
        serde_json::to_string(self).map_err(|x| { () })
    }

    pub fn default() -> Config {
       // let http_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080);
        Config {
            http_listen_address: String::from("127.0.0.1:8080"),
            knx_server: None,
            knx_multicast_group: Ipv4Addr::UNSPECIFIED,
            knx_multicast_interface: Ipv4Addr::UNSPECIFIED,
            knx_multicast_port: 3086,
            room_list: vec![],
            rooms: HashMap::new(),
            sensors: HashMap::new(),
            actors: HashMap::new(),
        }
    }

    pub fn read(path: &String) -> Result<Config, String> {
        let content = std::fs::read_to_string(path)
            .expect("Could not read file at {path}");

        let mut config: Config = serde_json::from_str(&content)
            .expect(":-(");

        Ok(config)
    }

    pub fn print(&self) {
        // change default values stored in configbuilder if they are available in config-file
        println!("from json:\n{:?}", self);
        for (id, room) in &self.rooms {
            println!("room '{id}': {room:?}");
        }
        for (id, sensor) in &self.sensors {
            println!("sensor '{id}': {sensor:?}");
        }
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
