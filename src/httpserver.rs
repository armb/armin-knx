extern crate handlebars;
extern crate chrono;

use std::string::String;

use scanf::sscanf;
use std::sync::{Arc, Mutex};
use std::net::{Ipv4Addr, SocketAddr};
use std::convert::Infallible;
use std::fmt::Error;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use handlebars::{Handlebars};
use hyper::{Request, Response};
use hyper::body::{Body};
use hyper::header;
use hyper::service::{make_service_fn, service_fn};

use crate::{config, Config, data};
use crate::data::{Data};

use serde_json::value::{Map, Value as Json};
use serde_json::json;
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

enum _Actor { NONE, ON, OFF, DIMMER { percent: u8 } }


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
                        let _unit_string = match sensor.dimension.as_str() {
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
                // println!("Actor {0} in {1}: {2:?}", actor_id, Actor.room_id, Actor.commands);
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
                // println!("Actor {0} in {1}: {2:?}", actor_id, Actor.room_id, Actor.commands);
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
                // check if known command for Actor
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

            let message = match Command::from_str(&command_string) {
                Ok(command) => {
                    println!("SENDCOMMAND:  command={command:?} to {addr:?}");
                    match h.knx.send(&addr.unwrap(), &command) {
                        Ok(_) => HttpServer::response_message(Ok("sent command".to_string())),
                        Err(text) => HttpServer::response_message(Err(text)),
                    }
                },
                Err(text) => HttpServer::response_message(Err(text))
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
//        let addr_str = INSTANCE.clone().unwrap().lock().unwrap().config.http_listen_address.clone();
        //let addr = SocketAddr::from_str(&addr_str).expect("could not parse {addr_str} as SocketAddrV4");

        let addr: SocketAddr = (Ipv4Addr::new(127, 0, 0, 1), 8081).into();
        println!("httpserver-address: {addr:?}");

       // let a = httpserver.clone().unwrap(); //arc
        let make_svc = make_service_fn(move |_| {
            //let remote_addr = socket.remote_addr();
            println!("make_service_fn: A");
            let service= service_fn(|request: Request<Body>| async move {
                Ok::<_, Infallible>({
                    //println!("service_fn: A");
                    //let mut h = INSTANCE.unwrap().clone().lock().unwrap();
                    //println!("service_fn: C");
                    match request.uri().path() {
                    //    "/test" => HttpServer::create_plot_response(request).unwrap(),
                        _other => HttpServer::create_response(request)
                    }
                })
            });
            async move { Ok::<_, Infallible>(service) }
        });

        let server =
            hyper::Server::bind(&addr).serve( make_svc);

        server.await.expect("Server failure");

        Ok(())
    }
}
