extern crate handlebars;

// #[macro_use]
// extern crate serde_json;

use handlebars::Handlebars;
use crate::config;
use std::sync::{Arc, Mutex};
use crate::config::Config;

#[derive(Debug)]
pub struct Html<'a> {
    // register
    handlebars: handlebars::Handlebars<'a>, // = handlebars::Handlebars::new(), //Option<handlebars::registry::Registry>,
    data: Data
}


#[derive(serde::Serialize,Debug,Clone)]
struct ItemData {
    id: String,
    name: String,
    // item: config::Item
}


#[derive(serde::Serialize,Debug)]
struct RoomData {
    id: String,
    name: String,
    items: Vec<ItemData>
}


#[derive(serde::Serialize,Debug)]
struct Data {
    title: String,
    rooms: Vec<RoomData>
}


#[derive(PartialEq)]
pub enum What {
    Index,
    JavaScript,
    SensorData
}


impl Html<'_> {
    pub fn render(&self, what: What) -> String {

        let a = Handlebars::new();
        match what {
            What::JavaScript =>
                self.handlebars.render("tpl_javascript", &String::from("")).expect("render()"),
            What::SensorData => {
                "".into()
            },
            _ => self.handlebars.render("tpl_index", &self.data).expect("render()")
        }
    }
}


pub fn create(config: Arc<Config>) -> Result<Html<'static>,String> {
    let mut reg = Handlebars::new();


    let mut data = Data { title: "TITLE".to_string(), rooms: Vec::new() };
    for (room_id, room) in config.rooms.iter() {
	let mut r = RoomData { id: room_id.to_string(), name: room, items: Vec::new() };

	// add items in this room
	for (item_id, item) in config.file.items.iter() {
	    if item.room != *room_id { continue; }

	    let i = ItemData { id: item_id.to_string(), name: item.name.clone(), item: item.clone() };
	    
	    r.items.push (i.clone());
	}

	data.rooms.push( r );
    }


  //  reg.register_template_string("tpl_1", "Good afternoon, {{name}}").expect("register_template_string()");

    // reg.register_templates_directory();
  //  reg.register_template_file("tpl_index", config.base_dir.to_string() + "res/tpl/index.html").expect("register");
  //  reg.register_template_file("tpl_javascript", config.base_dir.to_string() + "res/tpl/javascript.js").expect("register");

    let html: Html = Html { handlebars: reg, data: data };

    Ok(html)
}
