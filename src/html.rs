extern crate handlebars;

// #[macro_use]
// extern crate serde_json;

use std::collections::BTreeMap;
use handlebars::Handlebars;
use crate::config;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::config::Config;


#[derive(Debug)]
pub struct Html<'a> {
    // register
    handlebars: handlebars::Handlebars<'a>, // = handlebars::Handlebars::new(), //Option<handlebars::registry::Registry>,
    config: Arc<Config>
}


// #[derive(serde::Serialize,Debug,Clone)]
// struct ItemData {
//     id: String,
//     name: String,
//     // item: config::Item
// }

//
// #[derive(serde::Serialize,Debug)]
// struct RoomData {
//     id: String,
//     name: String,
//     items: Vec<ItemData>
// }
//
//
// #[derive(serde::Serialize,Debug)]
// struct Data {
//     title: String,
//     rooms: Vec<RoomData>
// }


#[derive(PartialEq)]
pub enum What {
    Index,
    JavaScript,
    SensorData
}


impl Html<'_> {
    pub fn render(&self, what: What) -> Result<String, ()> {
        match what {
//            What::JavaScript => self.render_javascript(),
//            What::SensorData => self.render_sensordata(),
            _ => self.render_index(),
        }
    }

    fn render_index(&self) -> Result<String, ()> {
        for (name, room) in &self.config.rooms {}
        // let mut data = BTreeMap::new();

        #[derive(Serialize)]
        struct Data {
            title: String,
            rooms: Vec<Room>,
        };
        #[derive(Serialize)]
        struct Room {
            floor: String,
            name: String,
            actions: Vec<Action>
        };
        #[derive(Serialize)]
        struct Action {
            text: String,
            id: String
        };

        let data = Data {
            title: "TITLE".to_string(),
            rooms: vec![
                Room {
                    floor: "EG".to_string(), name: "Flur".to_string(),
                    actions: vec![
                        Action {
                            text: "Licht aus".to_string(),
                            id: "eg.wohn.licht.aus".to_string()
                        },
                        Action {
                            text: "Licht an".to_string(),
                            id: "eg.wohn.licht.an".to_string(),
                        }
                    ]},
                Room {
                    floor: "EG".to_string(), name: "Bad".to_string(),
                    actions: vec![
                        Action {
                            text: "Licht aus".to_string(),
                            id: "eg.bad.licht.aus".to_string()
                        },
                        Action {
                            text: "Licht an".to_string(),
                            id: "eg.bad.licht.an".to_string(),
                        }
                    ]},
            ]
        };

        // data.insert("title".to_string(), "TITLE".to_string());
        // data.insert("rooms".to_string(), &a);
        let str = self.handlebars.render("tpl_index", &data).expect("render()");

        Ok(str)
    }
}


pub fn create(config: Arc<Config>) -> Result<Html<'static>,String> {
    let mut handlebars = Handlebars::new();

    handlebars.register_template_file("tpl_index", "res/tpl/tpl_index.html");

    let html = Html { handlebars, config };
    Ok(html)
}
  //   let mut data = Data { title: "TITLE".to_string(), rooms: Vec::new() };
  //   for (room_id, room) in config.rooms.iter() {
	//     let mut r = RoomData {
  //           id: room_id.to_string(),
  //           name: room,
  //           items: Vec::new() };
  //
  //       // add items in this room
  //       for (item_id, item) in config.file.items.iter() {
  //           if item.room != *room_id { continue; }
  //
  //           let i = ItemData { id: item_id.to_string(), name: item.name.clone(), item: item.clone() };
  //           r.items.push (i.clone());
  //       }
  //
  //   	data.rooms.push( r );
  //   }
  //
  // //  reg.register_template_string("tpl_1", "Good afternoon, {{name}}").expect("register_template_string()");
  //
  //   // reg.register_templates_directory();
  // //  reg.register_template_file("tpl_index", config.base_dir.to_string() + "res/tpl/index.html").expect("register");
  // //  reg.register_template_file("tpl_javascript", config.base_dir.to_string() + "res/tpl/javascript.js").expect("register");