extern crate handlebars;
extern crate chrono;
extern crate plotters;

use std::string::String;

use scanf::sscanf;
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, LinkedList};
//use hyper::server::Server;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::convert::Infallible;
use std::fmt::Error;
use std::fs::File;
use std::io::Read;
use std::ops::Add;
use std::str::FromStr;
use std::{os, time};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use handlebars::{to_json, Handlebars};
use hyper::{Body, Request, Response, Server};
use hyper::body::HttpBody;
use hyper::server::conn::AddrStream;
use hyper::header;
use hyper::http::{HeaderValue};
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use plotters::backend::{BitMapBackend, SVGBackend};
use plotters::coord::Shift;
use plotters::prelude::IntoDrawingArea;
use plotters::series::AreaSeries;
use plotters::style::{BLUE, Color, RED};
use tokio::net::TcpListener;
// use hyper::server::conn::AddrStream;
// use serde_json::Value::String;
use crate::{config, Config, data, knx};
use crate::data::{Data, Dimension};
use crate::config::{Room};

use serde_json::value::{Map, Value as Json};
use serde_json::json;
// use serde_json::Value::String;
use crate::knx::{Command, KnxSocket};


#[derive()]
pub struct HttpServer {
    config: Arc<config::Config>,
    data: Arc<Mutex<data::Data>>,
    knx: KnxSocket,



}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TemplateSensor { id: String, dimension: String, name:  String, measurement:  String, timestamp: String }
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TemplateActor { id:  String, name:  String, status: String, commands: Vec<String> }
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TemplateSwitch { id:  String, name:  String, status: String, commands: Vec<String> }
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TemplateRoom {
    id: String,
    name: String,
    actors: Vec<TemplateActor>,
    sensors: Vec<TemplateSensor>,
    switches: Vec<TemplateSwitch>,
}

static mut INSTANCE: Option<Arc<Mutex<HttpServer>>> = None;

enum Actor { NONE, ON, OFF, DIMMER { percent: u8 } }


impl HttpServer {
    pub async unsafe fn create(config: Arc<config::Config>, data: Arc<Mutex<data::Data>>) -> Result<(), String> {
        // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut knx = KnxSocket::create().unwrap();
        if let Some(server) = &config.knx_server {
            knx.connect(server).expect(format!("knx connect to {server} failed").as_str());
        }

        // let mut drawing_area = BitMapBackend::new("images/drawing_area.png", (1024,768))
//            .into_drawing_area();

        let s = HttpServer { config, data, knx };


        //httpserver = Some(Arc<Mutex<HttpServer>>)
        unsafe {
             INSTANCE = Some( Arc::new(Mutex::new(s)) )
        };

        Ok ( () )
    }


    fn create_plot_response(_req: Request<Body>) -> Result<Response<Body>, Infallible> {


        //let mut svg: String = "uninitialized".to_string();
        //let mut drawing_area = plotters::backend::SVGBackend::with_string(&mut svg, (100, 100)).into_drawing_area();

        let drawing_area = BitMapBackend::new("2.png", (600, 400))
            .into_drawing_area();

        drawing_area.fill(&plotters::prelude::WHITE).unwrap();


        let mut chart = plotters::prelude::ChartBuilder::on(&drawing_area)
             .build_cartesian_2d(-1.0..11.0, -2.0.. 30.0)
             .unwrap();

        let data = [(0,25.1), (1,37.2), (2,15.3), (3,32.4), (4,45.1), (5,33.6), (6,32.4), (7,10.3), (8,29.8), (9,0.9), (10,21.2)];

        chart.draw_series(
            AreaSeries::new(data.map(|(x,y)| (x as f64,y)),
                            0.,
                            BLUE.mix(0.2)).border_style(BLUE)).unwrap();
        // chart.draw_series(
        //     AreaSeries::new(
        //         (0..).zip(data.iter().map(|x| *x)), // The data iter
        //         0.0,                                  // Baseline
        //         &RED.mix(0.2) // Make the series opac
        //     ).border_style(&RED) // Make a brighter border
        // )
        //     .unwrap();

        drawing_area.present().expect("TODO: panic message");

       // println!("svg: {:?}", &mut svg);

       let plot = std::fs::read("2.png").expect("2.png");
        let response = Response::builder()
            .header("Content-type", "image/png")
            .body(Body::from(plot)).expect("response");
        Ok(response)
    }


    fn response_message(message: Result<String, String>) -> Response<Body> {
        match message {
            Ok(text) => {
                Response::builder().status(200).body(Body::from(text)).unwrap()
            },
            Err(text) => {
                Response::builder().status(400).body(Body::from(text)).unwrap()
            }
        }
    }

    fn create_response_error(_request: &Request<Body>, message: String) -> Response<Body> {
        return Response::builder().status(400).body(Body::from(message)).unwrap()
    }


    fn template_data_for_room(room_id: &String, config: &Arc<Config>, data: &Arc<Mutex<Data>>) -> Result<TemplateRoom, String> {
        let config_room = config.rooms.get(room_id)
            .expect("Raum nicht gefunden");

        let mut room_sensors: Vec<TemplateSensor> = vec![];
        for (sensor_id, sensor) in &config.sensors {
            if sensor.room_id.eq(room_id) {
                let data = data.lock().unwrap();
                let mut template_sensor = TemplateSensor {
                    id: sensor_id.clone(),
                    name: sensor.name.clone(),
                    dimension: sensor.dimension.clone(),
                    measurement: "".to_string(),
                    timestamp: "".to_string(),
                };
                let data_measurement = data.measurements.get(sensor_id);
                template_sensor.timestamp = match data_measurement {
                    Some(m) => {
                        if m.value.is_some() {
                            let datetime: chrono::DateTime<chrono::Local> = m.timestamp.into();
                            datetime.format("%T").to_string()
                        } else {
                            "".to_string()
                        }
                    },
                    _ => "".to_string()
                };
                template_sensor.measurement = match data_measurement {
                    Some(measurement) => {
                        let unit_string = match sensor.dimension.as_str() {
                            "temperature" => "°C",
                            "brightness" => "lux",
                            "onoff" => "",
                            _ => "?"
                        };
                        if let Some(value) = measurement.value {
                            format!("{value:.1}")
                        } else {
                            "?".to_string()
                        }
                    },
                    None => "?".to_string()
                };
                room_sensors.push(template_sensor);
            }
        }
        let mut room_actors: Vec<TemplateActor> = vec![];
        for (actor_id, actor) in  &config.actors {
            if actor.room_id.eq(room_id) {
                // println!("actor {0} in {1}: {2:?}", actor_id, actor.room_id, actor.commands);
                let template_actor = TemplateActor {
                    id: actor_id.clone(),
                    name: actor.name.clone(),
                    status: "".to_string(),
                    commands: actor.commands.clone()
                };
                room_actors.push(template_actor);
            }
        }
        let mut room_switches: Vec<TemplateSwitch> = vec![];
        for (switch_id, switch) in  &config.switches {
            if switch.room_id.eq(room_id) {
                // println!("actor {0} in {1}: {2:?}", actor_id, actor.room_id, actor.commands);
                let template_switch = TemplateSwitch {
                    id: switch_id.clone(),
                    name: switch.name.clone(),
                    status: "".to_string(),
                    commands: switch.commands.clone()
                };
                room_switches.push(template_switch);
            }
        }
        let room = TemplateRoom {
            id: room_id.clone(),
            name: config_room.name.clone(),
            sensors: room_sensors,
            actors: room_actors,
            switches: room_switches };

        Ok( room )
    }

    unsafe fn create_response(request: Request<Body>) -> Response<Body> {

        println!("--- REQUEST: {request:?}");

        let binding = INSTANCE.clone().unwrap();
        let mut h = binding.lock().unwrap();



        let path = request.uri().path();
        let mut set_id =  String::new();
        let mut command_string =  String::new();
        let mut static_path = String::new();
        if sscanf!(path, "static/{}", static_path).is_ok() {
            println!("STATIC: '{static_path}'");
            let static_rel = static_path.strip_prefix('/').unwrap();
            println!("STATIC-rel: '{static_rel}'");
            let mut buf = Vec::new();
            let content_type = if path.ends_with(".png") {
                "image/png"
            } else if path.ends_with(".css") { "text/css"
            } else if path.ends_with(".js") { "text/javascript"
            } else { "unknown" };
            return if let Ok(mut file) = File::open(static_rel) {
                let message = match file.read_to_end(&mut buf) {
                    Ok(len) => {
                        println!("OK: size={len}");
                        let body = Body::from(buf);
                        Response::builder()
                            .header(header::CONTENT_TYPE, content_type)
                            .status(hyper::StatusCode::OK)
                            .body(body).unwrap()
                    },
                    _ => {
                        println!("Error");
                        HttpServer::response_message(Ok("read error".to_string()))
                    }
                };
                message
            } else {
                println!("Not found");
                HttpServer::response_message(Ok("file not found".to_string()))
            }
        }
        if sscanf!(path, "/actor/{}/{}", set_id, command_string).is_ok() {
            println!("SET:  id={set_id} -> {command_string}");
            let mut addr = None;
            if let Some(actor) = h.config.actors.get(&set_id) {
                // check if known command for actor
                if actor.commands.contains(&command_string) {
                    addr = Some(actor.eibaddr.clone());
                }
            } else if let  Some(switch) = h.config.switches.get(&set_id) {
                if switch.commands.contains(&command_string) {
                    // check if known command for switch
                    addr = Some(switch.eibaddr_command.clone());
                }
            }

            if addr == None {
                return Self::create_response_error(
                    &request,
                    format!("no actor/switch found for command '{command_string}'").to_string());
            };

            let command = match command_string.as_str() {
                "on" => Command::Switch(true),
                "off" => Command::Switch(false),
                "dim-0" => Command::Dimmer(0),
                "dim-5" => Command::Dimmer(5),
                "dim-10" => Command::Dimmer(10),
                "dim-25" => Command::Dimmer(25),
                "dim-50" => Command::Dimmer(50),
                "dim-100" => Command::Dimmer(100),
                "shutter-0" => Command::Shutter(1),
                "shutter-50" => Command::Shutter(50),
                "shutter-90" => Command::Shutter(90),
                "shutter-170" => Command::Shutter(170),
                "shutter-180" => Command::Shutter(180),
                "shutter-255" => Command::Shutter(255),
                _ => {
                    return Self::create_response_error(
                        &request,
                        format!("unknown command '{command_string}'").to_string());
                },
            };

            println!("SENDCOMMAND:  command={command:?} to {addr:?}");
            let message = match h.knx.send(&addr.unwrap(), &command) {
                Ok(_) => HttpServer::response_message(Ok("sent command".to_string())),
                Err(text) => HttpServer::response_message(Err(text)),
            };

            return message;
        }
        let response = match path {
            "/style.css" => {
                let mut handlebars = Handlebars::new();
                handlebars.set_dev_mode(true);
                handlebars.register_template_file("style", "res/tpl/style.css")
                    .expect("should register style template file");
                //     .expect("Could not register template file for 'style'");
                let template_values = Map::< String, Json>::new();
                //
                // template_values.insert("rooms".to_string(), json!(template_rooms));
                let content = handlebars.render("style", &template_values).unwrap();
                //let content = "Dies ist ein Test.".to_string();

                let body = Body::from(content);
                Response::builder()
                    .header(header::CONTENT_TYPE, "text/plain")
                    .status(hyper::StatusCode::OK)
                    .body(body)
            }
            "/" => {
                let mut template_rooms: Vec<TemplateRoom> = vec![];
                for room_id in &h.config.room_list {
                    if ! h.config.rooms.contains_key(room_id) {
                        eprintln!("Details zu Raum {room_id} nicht in Konfiguration gefunden.");
                    }
                    let room = HttpServer::template_data_for_room(room_id, &h.config, &h.data)
                        .expect("Could not find room details in config");
                    template_rooms.push( room );
                }

                let mut handlebars = Handlebars::new();
                handlebars.set_dev_mode(true);
                handlebars.register_template_file("index", "res/tpl/tpl_index.html")
                    .expect("Could not register template file for 'index'");

                let mut template_values = Map::< String, Json>::new();
                //template_values.insert()
                template_values.insert("title".to_string(), json!(""));
                template_values.insert("rooms".to_string(), json!(template_rooms));
                let content = handlebars.render("index", &template_values).unwrap();

                let body = Body::from(content);

                Response::builder()
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .status(hyper::StatusCode::OK)
                    .body(body)
            },
            _ => {
                let body = Body::from("ERROR");

                Response::builder()
                    .header(header::CONTENT_TYPE, "text/plain")
                    .status(hyper::StatusCode::NOT_FOUND)
                    .body(body)
            }
        }.expect("Could not create response");

        response
    }


    pub async unsafe fn thread_function() -> Result<(), Error> {

        //let c = httpserver.lock().expect("httpserver");
        //let addr_str = INSTANCE.clone().unwrap().lock().unwrap().config.http_listen_address.clone();
        //let addr = SocketAddr::from_str(&addr_str).expect("could not parse {addr_str} as SocketAddrV4");


        let addr: SocketAddr = (Ipv4Addr::new(0, 0, 0, 0), 8081).into();
        println!("httpserver-address: {addr:?}");

       // let a = httpserver.clone().unwrap(); //arc
        let make_svc = make_service_fn(move |_socket:&AddrStream| {
            //let remote_addr = socket.remote_addr();
            println!("make_service_fn: A");
            let service= service_fn(|request: Request<Body>| async move {
                Ok::<_, Infallible>({
                    //println!("service_fn: A");
                    //let mut h = INSTANCE.unwrap().clone().lock().unwrap();
                    //println!("service_fn: C");
                    match request.uri().path() {
                        "/test" => HttpServer::create_plot_response(request).unwrap(),
                        _other => HttpServer::create_response(request)
                    }
                })
            });
            async move { Ok::<_, Infallible>(service) }
        });


        let server =
            Server::bind(&addr).serve( make_svc);

        server.await.expect("Server failure");

        Ok(())
    }
}
