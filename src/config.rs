use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EibAddr(u8, u8, u8);


pub fn parse_addr(s: &str) -> Result<EibAddr,Error> {
    // haupt/mittel/untergruppe
    let re = regex::Regex::new(r"^(?P<haupt>[[:digit:]]+)/(?P<mittel>[[:digit:]]+)/(?P<unter>[[:digit:]]+)$").unwrap();
    match re.captures(s) {
	Some(cap) => {
	    let a = EibAddr(
		cap["haupt"].parse::<u8>().unwrap(),
		cap["mittel"].parse::<u8>().unwrap(),
		cap["unter"].parse::<u8>().unwrap());

	    Ok(a)
	},
	None => {
	    Err( Error::GenericError( format!("address '{}' does not match format 'x/y/z'", s) ))
	}
    }
}


#[derive(Serialize,Deserialize,Debug)]
pub struct ConfigFile {
    pub title: String,
    pub version: String,
    pub http: Http,
    pub rooms: HashMap<String,Room>,
    pub items: HashMap<String,Item>
}


#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    GenericError(String),
    NotFound
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
	Error::ParseError(err)
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
	Error::IoError(err)
    }
}


#[derive(PartialEq,Serialize,Deserialize,Debug,Clone)]
pub enum KnxType {
    Switch,
    Dimmer,
    Bell,
    DoorOpener,
    Blinds
}


#[derive(Serialize,Deserialize,Debug)]
pub struct Http {
    pub listen_port: u16
}


#[derive(Serialize,Deserialize,Debug)]
pub struct Room {
    pub name: String
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Item {
    pub name: String,
    pub room: String,
    pub icon: Option<String>,
    pub knx_type: KnxType,
    pub knx_write_group: Option<String>,
    pub knx_read_group: Option<String>,
}


#[derive(Debug)]
pub struct Config {
    pub file: ConfigFile
}


pub fn read_from_file(path: &str) -> Result<Config,Error> {

    let config_file_data = std::fs::read_to_string(path)?;

    let c: ConfigFile = serde_json::from_str(&config_file_data)?;

    Ok (Config { file: c } )
}


impl Config {
    pub fn get_write_addr(&self, id: &str) -> Result<EibAddr,Error> {
	match self.file.items.get(id) {
	    // not in database
	    None => Err(Error::NotFound),

	    // found item
	    Some(item) => {
		match &item.knx_write_group {
		    // item has no address
		    None => Err(Error::NotFound),
		    Some(addr_string) => parse_addr(&addr_string)
		}
	    }
	}
    }
}
