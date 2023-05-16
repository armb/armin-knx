use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, LinkedList};
//use hyper::server::Server;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::convert::Infallible;
use std::ops::Add;
use std::str::FromStr;
use handlebars::Handlebars;
use hyper::{Body, Request, Response, Server};
use hyper::header;
use hyper::http::{Error, HeaderValue};
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use tokio::net::TcpListener;
// use hyper::server::conn::AddrStream;
// use serde_json::Value::String;
use crate::{config, Config, data};
use crate::data::Data;
use crate::httpserver::Action::{OFF, ON};

pub struct HttpServer {
    config: Arc<config::Config>,
    data: Arc<Mutex<data::Data>>,
}

enum Action { NONE, ON, OFF, DIMMER { percent: u8 } }


// https://hyper.rs/guides/server/echo/


impl HttpServer {
    pub fn create(config: Arc<config::Config>, data: Arc<Mutex<data::Data>>) -> HttpServer {
        // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let result = HttpServer { config, data };

        // demo:
        let req = hyper::Request::builder()
            .uri("http://localhost")
            .header("X-Test", "TEST")
            .body(Default::default()).unwrap();
        let response = result.create_response(
            req,
            &Default::default(),
            Arc::new(Mutex::new(Data::new())))
            .unwrap();
        println!("Response= {response:?}");

        result
    }


    async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new(Body::from("Hello World")))
    }

    fn create_response(&self,
              request: Request<Body>) -> Result<Response<Body>, Infallible> {
        let data = self.data.clone().lock().unwrap();

        println!("---- Request: {request:?}");
        //let mut a = form_urlencoded::parse(req.uri().query().expect("query?").as_bytes());
        // if let l = a.find(|(a, b)| a == "action" ) {
        //     println!("action: {:?}", l);
        // }


        let r1 = config::Room { name: "AAA".to_string() };
        let r2 = config::Room { name: "BBB".to_string() };
        let rooms = [r1, r2];

        let path = request.uri().path();
        let template_name = match request.uri().path() {
            "/" => "index",
            _ => path,  // all others
        };
        println!("path: '{path}'");
        let mut msg = String::from("TEST:\n");
        msg.push_str(&format!("path: {path}\n"));
        for (id, m) in &data.measurements {
            msg.push_str(format!("{id}: {:?}\n", m.value).as_str());
        }


        let mut handlebars = Handlebars::new();
        handlebars.register_template_file("index", "res/tpl/tpl_index.html")
            .expect("Could not register template file for 'index'");

        let mut template_values = BTreeMap::new();
        //template_values.insert()
        template_values.insert("title".to_string(), "---- MEINE ÜBERSCHRIFT ----".to_string());
        let index = handlebars.render(template_name, &template_values).unwrap();
        // let b = unsafe {
        //     let h = handlebars.as_ref().unwrap();
        //     h.render("form", &data).unwrap()
        // };
        msg.push_str(index.as_str());
        // msg.push_str(&b);

        let body = Body::from(msg);

        let response = Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .status(hyper::StatusCode::OK)
            .body(body)
            .expect("Could not create response");

        Ok::<_, Infallible>(response)
    }


    pub async fn thread_function(&self) -> Result<(), ()> {
        let s = self.config.http_listen_address.as_str();
        let address = SocketAddrV4::from_str(s).unwrap();

        println!("httpserver-address: {address}");

        // https://docs.rs/hyper/latest/hyper/server/index.html

//         let service = hyper::service::make_service_fn();
//
//         let listener = TcpListener::bind(self.addr)
//             .await.expect("");
//
//         loop {
//             println!("loop {{");
//             let (stream, addr) = listener.accept()
//                 .await.expect("accept()");
//             println!(" accepted connection from {:?}", addr);

        let addr: SocketAddr = (Ipv4Addr::new(127, 0, 0, 1), 8080).into();
        let make_svc = make_service_fn(|socket:&hyper::server::conn::AddrStream| {
            let remote_addr = socket.remote_addr();
            async move  {
                Ok::<_, Infallible>(service_fn(move |request: Request<Body>| async move {
                    Ok::<_, Infallible>(
                        // self.create_response(request)
                        //     .expect("Could not create response for request {request:?}")
                        Response::new(Body::from(format!("Hello, {}!", remote_addr)))
                    )
                }))
            }
        });

        let server = Server::bind(&addr)
                 .serve( make_svc);

        server.await.expect("Server failure");
        Ok(())
    }
}
