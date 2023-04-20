use std::env;
use std::sync::{Arc, Mutex};
use tauri::{Builder, generate_handler, command};

mod knx;
mod config;
mod data;


type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


// on windows disable command prompt window for bundled apps
#[cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]
#[tokio::main]

async fn main() -> Result<()> {
    //
    // env::set_var("OUT_DIR", "TEMP");
    // assert_eq!(env::var("OUT_DIR"), Ok("TEMP".to_string()));
    //
    // include!(concat!(env!("OUT_DIR"), "/data.rs"));


    let config =
        Arc::new(
            config::ConfigBuilder::new()
                .read("res/config.json".into()).expect("read failed")
                .build().expect("config failed"));

    println!("config: {:?}", *config);

    let mut data = Arc::new(Mutex::new(data::Data::new()));

    let mut knx =
        knx::create(config, data.clone()).expect("create knx");

    let mut ui =
        tauri::Builder::default()
            .run(tauri::generate_context!());
            // .expect("error while running tauri application");
    //
    tokio::join!(
         knx.thread_function(),
         ui);

    Ok(())
}