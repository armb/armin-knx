use std::collections::{HashMap};
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
pub struct actor {
    pub name: String,
    pub room_id: String,
    pub commands: Vec<String>,
    pub eibaddr: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sensor {
    pub dimension: String,
    pub name: String,
    pub room_id: String,
    pub eibaddr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Switch {
    pub name: String,
    pub room_id: String,
    pub eibaddr_command: String,
    pub eibaddr_status: String,
    pub commands: Vec<String>,
}

impl Sensor {
    pub fn get_dimension(&self) -> Dimension {
        match self.dimension.as_str() {
            "brightness" => Dimension::Brightness,
            "temperature" => Dimension::Temperature,
            "onoff" => Dimension::OnOff,
            "percent" => Dimension::Percent,
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
    pub actors: HashMap<String, actor>,
    pub switches: HashMap<String, Switch>,
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
            switches: HashMap::new(),
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
        for (id, actor) in &self.actors {
            println!("actor '{id}': {actor:?}-")
        }
        for (id, switch) in &self.switches {
            println!("switch '{id}': {switch:?}-")
        }
    }
}
