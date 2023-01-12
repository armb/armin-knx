

use hyper::server::Server;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::convert::Infallible;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::http::Error;
use hyper::service::{make_service_fn, service_fn};
use crate::data;

pub struct HttpServer {
    addr: SocketAddr,
    data: Arc<Mutex<data::Data>>,
}


// https://hyper.rs/guides/server/echo/



impl HttpServer {
    // async fn handle(
    //     &self,
    //     addr: SocketAddr,
    //     req: Request<Body>) -> Result<Response<Body>, Infallible> {
    //         Ok(Response::new(Body::from("Hello World")))
    // }

    async fn html_body(&self, path: &String) -> Body {
        let mut content = String::default();

        let value = self.data.lock().expect("lock()").total_power.value;
        content += &*format!("Value: {}", value);
        Body::from(content)
    }

    // Implementing a Service when used with make_service_fn
    async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mut response = Response::new(Body::empty());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                *response.body_mut() = Body::from("Try POSTing data to /echo");
            },
            (&Method::POST, "/echo") => {
                // we'll be back
                *response.body_mut() = req.into_body();
            },
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            },
        };

        Ok(response)
    }

    pub fn create(data: Arc<Mutex<data::Data>>) -> HttpServer {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        HttpServer { addr, data }
    }


    pub async fn thread_function(&self) -> () {
        // And a MakeService to handle each connection...
        // let make_service = make_service_fn(|_conn| async {
        //     Ok::<_, Infallible>(service_fn(Self::hello_world ))
        // });


        // And a MakeService to handle each connection...
        let make_service = make_service_fn(|_| async {
            Ok::<_, Error>(service_fn(|_req| async {
                Ok::<_, Error>(Response::new(Body::from("Hello World")))
            }))
        });

        // Then bind and serve...
        let builder = Server::bind(&self.addr)
            .serve(make_service);


        // And run forever...
        if let Err(e) = builder.await {
            eprintln!("server error: {}", e);
        }
    }
}
