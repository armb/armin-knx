extern crate handlebars;

// #[macro_use]
// extern crate serde_json;

use handlebars::Handlebars;
use crate::config;

#[derive(Debug)]
pub struct Html {
    // register
    handlebars: Handlebars,
    data: Data
}


#[derive(serde::Serialize,Debug,Clone)]
struct ItemData {
    id: String,
    name: String,
    item: config::Item
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
    JavaScript
}


impl Html {
    pub fn render(&self, what: What) -> String {

	if what == What::JavaScript {
	    let r = self.handlebars.render("tpl_javascript", &String::from("")).expect("render()");
	    return r;
	}

	let r = self.handlebars.render("tpl_index", &self.data).expect("render()");

	r
    }
}


pub fn create(config: &config::Config) -> Result<Html,String> {
    let mut reg = Handlebars::new();


    let mut data = Data { title: "TITLE".to_string(), rooms: Vec::new() };
    for (room_id, room) in config.file.rooms.iter() {
	let mut r = RoomData { id: room_id.to_string(), name: room.name.clone(), items: Vec::new() };

	// add items in this room
	for (item_id, item) in config.file.items.iter() {
	    if item.room != *room_id { continue; }

	    let i = ItemData { id: item_id.to_string(), name: item.name.clone(), item: item.clone() };
	    
	    r.items.push (i.clone());
	}

	data.rooms.push( r );
    }


    reg.register_template_string("tpl_1", "Good afternoon, {{name}}").expect("register_template_string()");

    // reg.register_templates_directory();
    reg.register_template_file("tpl_index", config.base_dir.to_string() + "res/tpl/index.html").expect("register");
    reg.register_template_file("tpl_javascript", config.base_dir.to_string() + "res/tpl/javascript.js").expect("register");

    let html: Html = Html { handlebars: reg, data: data };

    Ok(html)
}
