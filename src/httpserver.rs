extern crate handlebars;

use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, LinkedList};
//use hyper::server::Server;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::convert::Infallible;
use std::ops::Add;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use handlebars::{to_json, Handlebars};
use hyper::{Body, Request, Response, Server};
use hyper::body::HttpBody;
use hyper::server::conn::AddrStream;
use hyper::header;
use hyper::http::{Error, HeaderValue};
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use tokio::net::TcpListener;
// use hyper::server::conn::AddrStream;
// use serde_json::Value::String;
use crate::{config, Config, data};
use crate::data::Data;
use crate::config::{Room};
use crate::httpserver::Action::{OFF, ON};

use serde_json::value::{Map, Value as Json};
use serde_json::json;


#[derive(Clone, Debug)]
pub struct HttpServer {
    config: Arc<config::Config>,
    data: Arc<Mutex<data::Data>>,
}

static mut httpserver: Option<Arc<HttpServer>> = None;

enum Action { NONE, ON, OFF, DIMMER { percent: u8 } }


impl HttpServer {
    pub fn create(config: Arc<config::Config>, data: Arc<Mutex<data::Data>>) -> Arc<HttpServer> {
        // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let s = Arc::new(HttpServer { config, data });
        unsafe {
            httpserver = Some(s.clone());
        }
        s
    }


    async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new(Body::from("Hello World")))
    }


    fn create_response(&self,
              request: Request<Body>) -> Result<Response<Body>, Infallible> {

        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct TemplateSensor { name: String, measurement: String };
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct TemplateActor { name: String, commands: Vec<String> };
        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct TemplateRoom {
            name: String,
            sensors: Vec<TemplateSensor>,
            actors: Vec<TemplateActor> };

        let mut template_rooms: Vec<TemplateRoom> = vec![];
        for room_id in &self.config.room_list {
            if ! self.config.rooms.contains_key(room_id) {
                eprintln!("Raum {room_id} nicht gefunden.");
            }
            let room = self.config.rooms.get(room_id)
                .expect("Raum nicht gefunden");
            let mut room_sensors: Vec<TemplateSensor> = vec![];
            for (sensor_id, sensor) in &self.config.sensors {
                if sensor.room_id.eq(room_id) {
                    println!("sensor in {0}: {1}", room.name, sensor_id);
                    let data = self.data.lock().unwrap();
                    let measurement = match data.measurements.get(sensor_id) {
                        Some(measurement) => { format!("...") },
                        None => "?".to_string()
                    };
                    let template_sensor = TemplateSensor {
                        name: sensor.name.clone(),
                        measurement
                    };
                    room_sensors.push(template_sensor);
                }
            }
            let mut room_actors: Vec<TemplateActor> = vec![];
            for (actor_id, actor) in  &self.config.actors {
                if actor.room_id.eq(room_id) {
                    println!("actor {0} in {1}: {2:?}", actor_id, actor.room_id, actor.commands);
                    let template_actor = TemplateActor {
                        name: actor.name.clone(),
                        commands: actor.commands.clone()
                    };
                    room_actors.push(template_actor);
                }
            }
            let template_room = TemplateRoom {
                name: room.name.clone(),
                sensors: room_sensors,
                actors: room_actors
            };
            template_rooms.push( template_room );
         }

        let response = match request.uri().path() {
            "/" => {
                let mut handlebars = Handlebars::new();
                handlebars.set_dev_mode(true);
                handlebars.register_template_file("index", "res/tpl/tpl_index.html")
                    .expect("Could not register template file for 'index'");

                let mut template_values = Map::<String, Json>::new();
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

        Ok::<_, Infallible>(response)
    }


    pub async unsafe fn thread_function() -> Result<(), ()> {

        let c = httpserver.clone().expect("httpserver");
        let addr_str = c.config.http_listen_address.clone();
        let addr = SocketAddr::from_str(&addr_str).expect("could not parse {addr_str} as SocketAddrV4");
        println!("httpserver-address: {addr:?}");

        //let addr: SocketAddr = (Ipv4Addr::new(0, 0, 0, 0), 8080).into();

        let make_svc = make_service_fn(|socket:&AddrStream| {
            //let remote_addr = socket.remote_addr();
            let service= service_fn(move |request: Request<Body>| async move {
                Ok::<_, Infallible>({
                    let c = httpserver.clone().expect("httpserver");
                    c.create_response(request).expect("create_response() failed")
                    // self.create_response(request)
                    //     .expect("Could not create response for request {request:?}")
                    //Response::new(Body::from(format!("Hello, {}!", "SAD"))) //remote_addr)))
                })
            });
            async move  { Ok::<_, Infallible>(service) }
        });


        let server =
            Server::bind(&addr).serve( make_svc);

        server.await.expect("Server failure");

        Ok(())
    }
}
