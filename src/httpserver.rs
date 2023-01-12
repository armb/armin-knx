

use hyper::server::Server;
use std::sync::{Arc, Mutex};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::convert::Infallible;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::http::Error;
use hyper::service::{make_service_fn, service_fn};
use hyper::server::conn::AddrStream;
use crate::data;
use crate::data::Data;

pub struct HttpServer {
    addr: SocketAddr,
    data: Arc<Mutex<data::Data>>,
}


// https://hyper.rs/guides/server/echo/



impl HttpServer {

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
        async fn create_response(req: Request<Body>, data: Arc<Mutex<Data>>) -> Result<Response<Body>, Infallible> {
            let d = data.lock().expect("");
            let flur = format!("{:?}", d.flur_brightness);
            let till = format!("{} {:?}", d.till.value, d.till.unit);

            let msg = format!("{flur}, {till}");
            Ok::<_, Infallible>(Response::new(Body::from(msg)))
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
