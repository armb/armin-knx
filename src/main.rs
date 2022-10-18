use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;

mod knx;
mod config;
mod data;
mod html;

#[tokio::main]
async fn main() {
    let config =
    Arc::new(
        config::ConfigBuilder::new()
            .read("res/config.json".into()).expect("read failed")
            .build().expect("config failed"));

    println!("config: {:?}", config.serialize().expect("serialize"));

    // let mut data = Arc::new(Mutex::new(data::Data::new()));

    let knx = knx::create(config.clone()).expect("KNX");

    let html = html::create(config.clone());

    let knx_thread = tokio::spawn(
        async move { knx.thread_function().await });

    //knx::thread_function().expect("udp thread");
    // knx.await.expect("knx await");
    // done

    knx_thread.await.expect("knx-thread stopped");
}