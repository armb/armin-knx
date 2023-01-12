extern crate handlebars;

// #[macro_use]
// extern crate serde_json;

use handlebars::Handlebars;
use std::sync::{Arc};
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
        // let mut data = BTreeMap::new();
        #[derive(Serialize)]
        struct Sensor {
            name: String,
            id: String
        }
        #[derive(Serialize)]
        struct Room {
            floor: String,
            name: String,
            actions: Vec<Action>,
            sensors: Vec<Sensor>,
        }
        #[derive(Serialize)]
        struct Action {
            text: String,
            id: String
        }

        #[derive(Serialize)]
        struct TemplateData {
            title: String,
            rooms: Vec<Room>,
        }
        let mut template_data = TemplateData {
            title: "".to_string(),
            rooms: vec![] };

        // add rooms from config to template data
        for (room_id, room) in &self.config.rooms {
            println!("----- Room {}", &room.name);
            let mut template_room = Room {
                floor: "?".to_string(),
                name: room.name.clone(),
                actions: vec![],
                sensors: vec![]
            };
            // look for sensors inside this room
            for (sensor_id, sensor) in &self.config.sensors {
                if sensor.room.eq(room_id) {
                    let template_sensor = Sensor { name: sensor.name.clone(), id: sensor_id.clone() };
                    template_room.sensors.push(template_sensor);
                }
            }
            template_data.rooms.push(template_room);
        }

        // data.insert("title".to_string(), "TITLE".to_string());
        // data.insert("rooms".to_string(), &a);
        let str = self.handlebars.render("tpl_index", &template_data).expect("render()");

        Ok(str)
    }
}


pub fn create(config: Arc<Config>) -> Result<Html<'static>,String> {
    let mut handlebars = Handlebars::new();

    handlebars
        .register_template_file("tpl_index", "res/tpl/tpl_index.html")
        .expect("could not read template file");
    //
    // let html = Html { handlebars, config };
    // Ok(html)

    Ok( Html{handlebars, config} )
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