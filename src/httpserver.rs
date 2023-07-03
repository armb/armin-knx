extern crate handlebars;

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
use serde::{Deserialize, Serialize};
use handlebars::{to_json, Handlebars};
use hyper::{Body, Request, Response, Server};
use hyper::body::HttpBody;
use hyper::server::conn::AddrStream;
use hyper::header;
use hyper::http::{HeaderValue};
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use tokio::net::TcpListener;
// use hyper::server::conn::AddrStream;
// use serde_json::Value::String;
use crate::{config, Config, data, knx};
use crate::data::{Data, Dimension};
use crate::config::{Room};
use crate::httpserver::actor::{OFF, ON};

use serde_json::value::{Map, Value as Json};
use serde_json::json;
// use serde_json::Value::String;
use crate::knx::{Command, KnxSocket};


#[derive(Debug)]
pub struct HttpServer {
    config: Arc<config::Config>,
    data: Arc<Mutex<data::Data>>,
    knx: KnxSocket
}

static mut instance: Option<Arc<Mutex<HttpServer>>> = None;

enum actor { NONE, ON, OFF, DIMMER { percent: u8 } }


impl HttpServer {
    pub async unsafe fn create(config: Arc<config::Config>, data: Arc<Mutex<data::Data>>) -> Arc<Mutex<HttpServer>> {
        // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut knx = KnxSocket::create().unwrap();
        if let Some(server) = &config.knx_server {
            knx.connect(server).expect("could not set knx message target: {server}");
        }

        //httpserver = Some(Arc<Mutex<HttpServer>>)
        let s = Arc::new(Mutex::new( HttpServer { config, data, knx } ) );
        unsafe {
            instance = Some(s.clone())
        };
        //httpserver = Some(s.clone());
        s
    }


    async fn handle(&self, _req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new(Body::from("Hello World")))
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

    unsafe fn create_response(request: Request<Body>) -> Response<Body> {

        println!("--- REQUEST: {request:?}");

        let binding = instance.clone().unwrap();
        let mut h = binding.lock().unwrap();
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct TemplateSensor { id: String, dimension: String, name:  String, measurement:  String };
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct TemplateActor { id:  String, name:  String, commands: Vec<String> };
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct TemplateRoom {
            id: String,
            name: String,
            actors: Vec<TemplateActor>,
            sensors: Vec<TemplateSensor>
        };

        let mut template_rooms: Vec<TemplateRoom> = vec![];
        for room_id in &h.config.room_list {
            if ! h.config.rooms.contains_key(room_id) {
                eprintln!("Details zu Raum {room_id} nicht in Konfiguration gefunden.");
            }
            let config_room = h.config.rooms.get(room_id)
                .expect("Raum nicht gefunden");
            let mut room_sensors: Vec<TemplateSensor> = vec![];
            for (sensor_id, sensor) in &h.config.sensors {
                if sensor.room_id.eq(room_id) {
                    let data = h.data.lock().unwrap();
                    let mut template_sensor = TemplateSensor {
                        id: sensor_id.clone(),
                        name: sensor.name.clone(),
                        dimension: sensor.dimension.clone(),
                        measurement: "".to_string(),
                    };
                    template_sensor.measurement = match data.measurements.get(sensor_id) {
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
            for (actor_id, actor) in  &h.config.actors {
                if actor.room_id.eq(room_id) {
                    // println!("actor {0} in {1}: {2:?}", actor_id, actor.room_id, actor.commands);
                    let template_actor = TemplateActor {
                        id: actor_id.clone(),
                        name: actor.name.clone(),
                        commands: actor.commands.clone()
                    };
                    room_actors.push(template_actor);
                }
            }
            let room = TemplateRoom {
                id: room_id.clone(),
                name: config_room.name.clone(),
                sensors: room_sensors,
                actors: room_actors};
            template_rooms.push( room );
         }

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
            let actor = h.config.actors.get(&set_id).expect("actor not found");
            let addr = actor.eibaddr.clone();
            // check if known command for actor
            if !actor.commands.contains(&command_string) {
                return Self::create_response_error(
                    &request,
                    format!("actor {set_id} has no command '{command_string}'").to_string());
            };

            let command = match command_string.as_str() {
                "on" => Command::Switch(true),
                "off" => Command::Switch(false),
                "dim-0" => Command::Dimmer(0),
                "dim-10" => Command::Dimmer(10),
                "dim-25" => Command::Dimmer(25),
                "dim-50" => Command::Dimmer(50),
                "dim-100" => Command::Dimmer(100),
                _ => {
                    return Self::create_response_error(
                        &request,
                        format!("unknown command '{command_string}'").to_string());
                },
            };

            println!("SENDCOMMAND:  command={command:?} to {addr}");
            let message = match h.knx.send(&addr, &command) {
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
                let mut template_values = Map::< String, Json>::new();
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
        let addr_str = instance.clone().unwrap().lock().unwrap().config.http_listen_address.clone();
        let addr = SocketAddr::from_str(&addr_str).expect("could not parse {addr_str} as SocketAddrV4");
        println!("httpserver-address: {addr:?}");

        //let addr: SocketAddr = (Ipv4Addr::new(0, 0, 0, 0), 8080).into();

       // let a = httpserver.clone().unwrap(); //arc
        let make_svc = make_service_fn(move |socket:&AddrStream| {
            //let remote_addr = socket.remote_addr();
            println!("make_service_fn: A");
            let service= service_fn(|request: Request<Body>| async move {
                Ok::<_, Infallible>({
                    //println!("service_fn: A");
                    //let mut h = instance.unwrap().clone().lock().unwrap();
                    //println!("service_fn: C");
                    let response = HttpServer::create_response(request);
                    //println!("service_fn: D");
                    response
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
