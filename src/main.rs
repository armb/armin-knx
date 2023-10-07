use std::borrow::Borrow;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::config::Config;
use httpserver::HttpServer;

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
    }

    // let content = "<!DOCTYPE html>\n".to_string()
    //     + html.render(html::What::Index).expect("html render error").as_str();
    //
    // std::fs::write("/tmp/out.html", content).expect("html render error");

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


    // describe the main window
    // let main_window = WindowDesc::new(build_root_widget())
    //     .title(WINDOW_TITLE)
    //     .window_size((400.0, 400.0));
    //
    // // create the initial app state
    // let initial_state = HelloState {
    //     name: "World".into(),
    // };
    //
    // // start the application
    // AppLauncher::with_window(main_window)
    //     .launch(initial_state)
    //     .expect("Failed to launch application");

    //tokio::join!(future_knx, future_httpserver);

    // tokio::join!(future_knx);

    Ok(())
}


fn build_root_widget() -> impl Widget<HelloState> {
    // a label that will determine its text based on the current app data.
    let label = Label::new(|data: &HelloState, _env: &Env| format!("Hello {}!", data.name));
    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(HelloState::name);

    // arrange the two widgets vertically, with some padding
    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox);

    // center the two widgets in the available space
    Align::centered(layout)
}