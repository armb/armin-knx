use std::collections::{BTreeMap, HashMap};
use hyper::server::Server;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::convert::Infallible;
use std::ops::Add;
use handlebars::Handlebars;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::body::HttpBody;
use hyper::header::CONTENT_TYPE;
use hyper::http::{Error, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::conn::AddrStream;
// use serde_json::Value::String;
use crate::data;
use crate::data::Data;
use crate::httpserver::Action::{OFF, ON};

pub struct HttpServer {
    addr: SocketAddr,
    data: Arc<Mutex<data::Data>>,
}



static mut handlebars: Option<Handlebars> = None;

enum Action { NONE, ON, OFF, DIMMER { percent: u8 } }



// https://hyper.rs/guides/server/echo/


impl HttpServer {

    pub unsafe fn create(data: Arc<Mutex<data::Data>>) -> HttpServer {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        if handlebars.is_none() {
            let mut h = Handlebars::new();
            h.register_template_file("index", "res/tpl/tpl_index.html").expect("ERROR");
            h.register_template_file("form", "res/tpl/tpl_form").expect("ERROR");

            handlebars = Some(h);
        }
        HttpServer { addr, data }
    }

    pub async fn thread_function(&self) -> () {
        // And a MakeService to handle each connection...
        // let make_service = make_service_fn(|_conn| async {
        //     Ok::<_, Infallible>(service_fn(Self::hello_world ))
        // });

        // And a MakeService to handle each connection...
        async fn create_response(req: Request<Body>, data: Arc<Mutex<Data>>) -> Result<Response<Body>, Infallible> {
            let d = data.lock().expect("");

            let mut a = form_urlencoded::parse(req.uri().query().expect("query?").as_bytes());
            if let l = a.find(|(a, b)| a == "action" ) {
                println!("action: {:?}", l);
            }
            // let action = a
            //     .find(|(k,_)| k.eq("action"))
            //
            //     .is
            //         "on" => Some(ON),
            //         "off" => Some(OFF),
            //         _ => None
            //     }).or(None);
            //
            // println!("ACTION: ---> {action:?}");

            let path = req.uri().path();
            let mut msg = String::from("TEST:\n");
            msg.push_str(&format!("path: {path}\n"));
            for (id, m) in &d.measurements {
                msg.push_str(format!("{id}: {:?}\n", m.value).as_str());
            }

            let mut data = BTreeMap::new();
            data.insert("title".to_string(), "Test".to_string());
            let a = unsafe {
                let h = handlebars.as_ref().unwrap();
                h.render("index", &data).unwrap()
            };
            let b = unsafe {
                let h = handlebars.as_ref().unwrap();
                h.render("form", &data).unwrap()
            };
            msg.push_str(&a);
            msg.push_str(&b);

            let body = Body::from(msg);

            let mut response = Response::new(body);
            response.headers_mut().insert(CONTENT_TYPE, "text/html".parse().unwrap());

            Ok::<_, Infallible>(response)
        };

        let data = self.data.clone();
        // create a MakeService from a function
        let make_svc = make_service_fn(move |_conn| { // outer closure
            let data = data.clone();
            async move { // async block
                Ok::<_, Infallible>(service_fn(move |_req| { // inner closure
                    create_response(_req, data.clone())
                }))
            }
        });

        // Then bind and serve...
        let builder = Server::bind(&self.addr)
            .serve(make_svc);


        // And run forever...
        if let Err(e) = builder.await {
            eprintln!("server error: {}", e);
        }
    }
}
