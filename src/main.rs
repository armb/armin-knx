use std::collections::LinkedList;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;
use crate::html::Html;

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

    let html = html::create(config.clone()).expect("HTML");

    let a = html.render(html::What::Index).expect("render index");

    println!("{}", a);

    let mut threads = Vec::new();
    match knx::create(config) {
        Ok(x) => { threads.push(tokio::spawn(async move { x.thread_function().await; })) },
        _ => ()
    }
    //knx::thread_function().expect("udp thread");g
    // knx.await.expect("knx await");
    // doneLinkedList; // <JoinHandle>::new();

    for handle in threads {
        handle.await.expect("thread terminated with Error")};
}