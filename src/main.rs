use std::sync::Arc;
mod knx;
mod config;
mod data;
mod html;
mod youless;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[tokio::main]
async fn main() -> Result<()> {
    let config =
    Arc::new(
        config::ConfigBuilder::new()
            .read("res/config.json".into()).expect("read failed")
            .build().expect("config failed"));

    println!("config: {:?}", *config);

    // let mut data = Arc::new(Mutex::new(data::Data::new()));

    let html = html::create(config.clone()).expect("HTML");

    let content = "<!DOCTYPE html>\n".to_string()
        + html.render(html::What::Index).expect("html render error").as_str();

    std::fs::write("/tmp/out.html", content).expect("html render error");


    let mut knx = knx::create(config).expect("create knx");
    let youless = youless::Youless::create();

    let future_knx =  knx.thread_function();
    let future_youless = youless.thread_function();

    tokio::join!(future_youless, future_knx);

    Ok(())
}