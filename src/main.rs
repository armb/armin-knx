use std::error::Error;
use std::sync::{Arc, Mutex};


use crate::config::Config;
use crate::knx::KnxSocket;


mod knx;
mod config;
mod data;
mod httpserver;
mod scheduler;


type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;


#[tokio::main]
async fn main() -> Result<(), > {
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

    let future_influxdb = || async {
       // sleep(time::Duration::from_secs(10));
         let client = influxdb::Client::new(
         "http://192.168.0.111:8086", "P3C6603E967DC8568");
        client.ping().await.expect("ping")
    };

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


    let mut scheduler_knx = KnxSocket::create().expect("knx socket");
    scheduler_knx.connect(&config.knx_server.clone().unwrap()).expect(format!("knx connect failed").as_str());
    let mut scheduler = scheduler::Scheduler::new(config.clone(), scheduler_knx).expect("read schedule");
    let future_scheduler = scheduler.thread_function();

//    tokio::spawn(future_scheduler);
   // future_influxdb().await.unwrap();
   // future_scheduler.await.unwrap();

    let _ = tokio::join!(future_httpserver(), future_knx, future_scheduler, future_influxdb());
    //});

    Ok(())
}
