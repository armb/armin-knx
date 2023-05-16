use std::sync::{Arc, Mutex};
use crate::config::Config;

mod knx;
mod config;
mod data;
mod httpserver;

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

    let mut data = Arc::new(Mutex::new(data::Data::new()));
    {
        let mut data = data.lock().unwrap();
        for (id, sensor) in &config.sensors {
            match data.add_sensor(&id, &sensor) {
                Ok(_) => println!("added sensor {id}"),
                Err(m) => eprintln!("ERROR: {m}")
            }
        }
    }

    // let content = "<!DOCTYPE html>\n".to_string()
    //     + html.render(html::What::Index).expect("html render error").as_str();
    //
    // std::fs::write("/tmp/out.html", content).expect("html render error");

    let mut knx = knx::create(
        config.clone(), data.clone()).expect("create knx");

    let mut h = handlebars::Handlebars::new();
    h.register_template_file("index", "res/tpl/tpl_index.html").expect("ERROR");

    let httpserver = httpserver::HttpServer::create(
        config.clone(), data.clone());

    let future_knx =  knx.thread_function();
    // let future_httpserver =  async { () }; //httpserver.thread_function();
    let future_httpserver = httpserver.thread_function();

    //tokio::join!(future_knx, future_httpserver);

    future_httpserver.await;
    // future_youless.await;
    // tokio::join!(future_youless, future_knx);

    Ok(())
}