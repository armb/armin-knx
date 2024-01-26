
use std::sync::{Arc, Mutex};

use crate::config::Config;


mod knx;
mod config;
mod data;
mod httpserver;

use druid::widget::{Align, Flex, Label, TextBox};
use druid::{AppLauncher, Data, Env, Lens, LocalizedString, Widget, WindowDesc, WidgetExt};

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<HelloState> = LocalizedString::new("Hello World!");

#[derive(Clone, Data, Lens)]
struct HelloState {
    name: String,
}


type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[tokio::main]
async fn main() -> Result<()> {
    let path = "res/config.json".to_string();
    let config =
        Arc::new(
            Config::read(&path)
                .expect("Could not read config file")
        );

    config.print();

    let data = Arc::new(Mutex::new(data::Data::new()));
    {
        let mut data = data.lock().unwrap();
        for (id, sensor) in &config.sensors {
            match data.add_sensor(&id, &sensor) {
                Ok(_) => println!("added sensor {id}"),
                Err(m) => eprintln!("ERROR: {m}")
            }
        }
        for (id, switch) in &config.switches {
            match data.add_switch(&id, switch) {
                Ok(_) => println!("added switch {id}"),
                Err(m) => eprintln!("ERROR: {m}")
            }
        }
    }

    let mut knx = knx::create(
        config.clone(), data.clone()).expect("create knx");

    let mut h = handlebars::Handlebars::new();
    h.register_template_file("index", "res/tpl/tpl_index.html").expect("ERROR");



    let future_knx =  knx.thread_function();
    // let future_httpserver =  async { () }; //httpserver.thread_function();
    let future_httpserver = || async {
        unsafe {
            httpserver::HttpServer::create(
            config.clone(), data.clone()).await.expect("TODO: panic message");
            // let mut a = httpserver.lock().unwrap();
            httpserver::HttpServer::thread_function().await.expect("TODO: panic message");
        };
    };


    //let _handle = thread::spawn(move || {
        let (_first, _second) = tokio::join!(future_httpserver(), future_knx);
    //});

    Ok(())
}
