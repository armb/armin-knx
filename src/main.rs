use std::collections::HashMap;
use std::convert::Infallible;
use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
// use std::time::{Duration, Instant};
use std::time::SystemTime;

use hyper::{Body, Request, Response, Server};
use hyper::body;
use tokio::task::JoinHandle;

use config::Config;
use html::Html;
use sensors::Wetter;

use crate::Measurement::{Brightness, Temperature};
use std::ops::Add;

mod knx;
mod config;
mod html;
mod sensors;

#[derive(Debug, Copy, Clone)]
pub struct Received { time: std::time::SystemTime, source: EibAddr, dest: EibAddr }
impl Received {
    pub fn _new() -> Self {
        Self {
            time: SystemTime::now(),
            source: EibAddr(0, 0, 0),
            dest: EibAddr(0, 0, 0)
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub enum Measurement {
    _Error,
    Undefined,
    Temperature(Received, f32), // Deg Celsiuspub
    Brightness(Received, f32), // Lux
}

#[derive(Debug, Copy, Clone)]
struct Sample {
    measurement: Measurement,
    timestamp: SystemTime
}


#[derive(Debug)]
struct Data { received: Received, hex_string: String }


#[derive(Debug, Copy, Clone, PartialEq)]
struct EibAddr(u8, u8, u8);

#[derive(Debug)]
struct HttpData {
    index: String,
    html: Html,
    uri_data: HashMap<String, Vec<u8>>,
}


// function never fails (always generates a Response<Body>)
fn http_get_request_handler(req: Request<Body>,
                            http_data: Arc<HttpData>,
                            remote_addr: SocketAddr,
                            wetter: Arc<Mutex<Wetter>>) -> Result<Response<Body>, Infallible> {

    // eprintln!("debug:  GET {:#?}", req);

    let mut handlebars = handlebars::Handlebars::new();

    for a in &[
        "index.html",
        "functions.js",
        "default-style.css",
    ] {
        let uri = format!("/{}", a);
        let path = format!("template/{}", a);
        handlebars.register_template_file(&uri, path).expect(format!("Could not register '{}'", a).as_str());
    }


    handlebars.register_template_file("/", "template/index.html").expect("Could not register root uri");

   //  let mut _w = wetter.lock().unwrap();
    // wetter.lock()a.push('.');

    //        _w.a.push('.');

    #[derive(serde::Serialize)]
    struct Info {
        title: std::string::String,
        bar: i64,
        addr: String,
        flur_brightness: String,
        till: String,
    }

    let info = Info {
        title: "Haus".to_owned(),
        bar: 1231,
        addr: format!("{:?}", remote_addr.to_string()),
        flur_brightness: "A".into(), //match _w.flur_brightness { Measurement::Brightness(_, t) => t.to_string(), _ => "".to_string() },
        till: "B".into() //match _w.till { Measurement::Temperature(_, t) => t.to_string(), _ => "".to_string() },
    };

    let uri = req.uri().path();

    if handlebars.has_template(&uri) {
        let output = handlebars.render(&uri, &info);
        if output.is_err() {
            eprintln!("GET '{:?}': could not render template", req.uri());
            Ok(Response::builder().status(hyper::StatusCode::NOT_FOUND)
                .header("Content-Type", "text/html; charset=utf8").body("Not found.".into()).unwrap())
        } else {
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "text/html; charset=utf8")
                .body(output.unwrap().into())
                .unwrap())
        }
    } else {
        match uri {
            "/sensors.json" => {
                let w = wetter.lock().expect("lock()");
                let mut t : String = "{\"a\":1".into();
                match w.flur_brightness {
                    Brightness(_, val) => { t.push_str( &format!(", \"eg.flur.brightness\":{:}", val)); },
                    _ => {}
                };
                match w.till {
                    Temperature(_, val) => { t.push_str( &format!(", \"og.till.temperature\":{:}", val)); },
                    _ => {}
                };
                t.push_str("}");
                let resp = Response::builder().status(hyper::StatusCode::OK).body(t.into());
                Ok(resp.unwrap())
            },
            _ => {
                // not found -> Nevertheless, return with Infallible to send a response to the client
                Ok(Response::builder().status(hyper::StatusCode::NOT_FOUND).body("Not found.".into()).unwrap())
            }
        }
    }
}


async fn http_request_handler(req: Request<Body>,
                              http_data: Arc<HttpData>,
                              remote_addr: SocketAddr,
                              wetter: Arc<Mutex<Wetter>>,
                              tx: Sender<KnxPacket>) -> Result<Response<Body>, Infallible> {
    let m = req.method().clone();

    if m == hyper::Method::PUT {
        let bytes = body::to_bytes(req.into_body()).await.expect("failed!");
        let body_str = String::from_utf8(bytes.to_vec()).expect("nody was not valid utf8");
        let a = KnxPacket { a: body_str };
        tx.send(a).expect("tx queue full");
    } else if m == hyper::Method::GET {
        return http_get_request_handler(req, http_data, remote_addr, wetter);
    }
    Ok(Response::builder().status(hyper::StatusCode::BAD_REQUEST).body("Bad request.".into()).unwrap())
}

fn _mythread(mut stream: TcpStream, _addr: SocketAddr) {
    eprintln!("New connection: {:?}", stream);
    // sleep(std::time::Duration::from_secs(1));
    stream.set_read_timeout(Option::from(std::time::Duration::from_secs(5))).expect("could not set read timeout");

    let a = std::string::String::from("ASD\n");

    stream.write_all(a.as_bytes()).expect("write failed");
    stream.write_all("Hallo\n\n".as_bytes()).expect("write failed");
    let r = std::io::BufReader::new(&stream);
    for l in r.lines() {
        println!("Zeile: {}", l.unwrap())
    }
}


#[derive(Debug)]
struct KnxPacket {
    a: String,
}


#[derive(Debug)]
enum Signal {
    // Error,
    OgTillLight,
    EgFlurSpots,
    EgKueche,

    EgWohnSpots,
    EgWohnMitte,
    EgWohnDosen,
    EgWohnDosen2,

    EgArbeitSpots,
    EgArbeitLight,
    EgArbeitSchreibtisch,
    EgArbeitDosen,
    EgArbeitRollo,

    EgEssenSpots,
    EgEssenDosen,

    EgWcLight,
    EgWohnRolloEinzel,
    EgWohnRolloDoppel,
    EgWohnDoseFenster,

    OgBadSpotsWarm,
    OgBadSpotsKalt,
    OgFlurSchrankzimmer,
    OgSchlafzimmer,

    Klingel,
    Summer
}



impl FromStr for Signal {
    type Err = ();
    fn from_str(input: &str) -> Result<Signal, Self::Err> {
        match input {
            "OgTillLight" => Ok(Signal::OgTillLight),
            "EgFlurSpots" => Ok(Signal::EgFlurSpots),
            "EgKueche" => Ok(Signal::EgKueche),

            "eg.wohn.spots" => Ok(Signal::EgWohnSpots),
            "EgWohnSpots" => Ok(Signal::EgWohnSpots),

            "EgWohnMitte" => Ok(Signal::EgWohnMitte),
            "EgArbeitSpots" => Ok(Signal::EgArbeitSpots),
            "EgArbeitLight" => Ok(Signal::EgArbeitLight),
            "EgArbeitSchreibtisch" => Ok(Signal::EgArbeitSchreibtisch),
            "EgArbeitDosen" => Ok(Signal::EgArbeitDosen),
            "EgArbeitRollo" => Ok(Signal::EgArbeitRollo),
            "EgEssenDosen" => Ok(Signal::EgEssenDosen),
            "EgEssenSpots" => Ok(Signal::EgEssenSpots),
            "EgWcLight" => Ok(Signal::EgWcLight),
            "EgWohnRolloEinzel" => Ok(Signal::EgWohnRolloEinzel),
            "EgWohnRolloDoppel" => Ok(Signal::EgWohnRolloDoppel),
            "EgWohnDoseFenster" => Ok(Signal::EgWohnDoseFenster),
            "eg.wohn.couch_dosen" => Ok(Signal::EgWohnDosen),
            "EgWohnDosen" => Ok(Signal::EgWohnDosen),
            "eg.wohn.tv" => Ok(Signal::EgWohnDosen2),
            "EgWohnDosen2" => Ok(Signal::EgWohnDosen2),
            "OgBadSpotsWarm" => Ok(Signal::OgBadSpotsWarm),
            "OgBadSpotsKalt" => Ok(Signal::OgBadSpotsKalt),
            "OgFlurSchrankzimmer" => Ok(Signal::OgFlurSchrankzimmer),
            "OgSchlafzimmer" => Ok(Signal::OgSchlafzimmer),
            "Klingel" => Ok(Signal::Klingel),
            "Summer" => Ok(Signal::Summer),
            _ => Err( () ),
        }
    }
}


#[derive(Debug)]
enum WebCommand {
    Error,
    Dimmer { signal: Signal, value: u8 },
    Switch { signal: Signal, value: bool },
    RolloWert { signal: Signal, value: u8 },
}


fn command_from_string( string: String) -> WebCommand
{
    println!(" command_from_string: {:X?}", string);

    let re_dimmer = regex::Regex::new(r"^Dimmer (?P<signal>[.[:word:]]+) (?P<value>[[:digit:]]+)$").unwrap();
    let caps_dimmer = re_dimmer.captures(&string);
    if let Some(cmd) = caps_dimmer {
        let signal = match Signal::from_str(&cmd["signal"]) { Ok(x) => x, _ => return WebCommand::Error };
        let value = match cmd["value"].parse::<u8>() { Ok(i) => i, Err(_) => return WebCommand::Error };
        println!("Dimmer: {:?} -> {:?}", signal, value);
        return WebCommand::Dimmer{ signal: signal, value: value };
    }

    let re_switch = regex::Regex::new(r"^Switch (?P<signal>[.[:word:]]+) (?P<value>1|0)$").unwrap();
    let caps_switch = re_switch.captures(&string);
    if let Some(cmd) = caps_switch {
        let signal = match Signal::from_str(&cmd["signal"]) { Ok(x) => x, _ => return WebCommand::Error };
        let value = if &cmd["value"] == "1" { true } else { false };
        println!("switch: {:?} : {:?}", &signal, &value);
        return WebCommand::Switch{ signal: signal, value: value };
    }

    let re_switch = regex::Regex::new(r"^RolloWert (?P<signal>[.[:word:]]+) (?P<value>[[:digit:]]+)%$").unwrap();
    let caps_switch = re_switch.captures(&string);
    if let Some(cmd) = caps_switch {
        let signal = match Signal::from_str(&cmd["signal"]) { Ok(x) => x, _ => return WebCommand::Error };
        let value = match cmd["value"].parse::<u8>() { Ok(i) => i, Err(_) => return WebCommand::Error };
        println!("rollo: {:?} : {:?}", &signal, &value);
        return WebCommand::RolloWert{ signal: signal, value: value };
    }

    WebCommand::Error
}

//function
fn bus_send_thread(rx: std::sync::mpsc::Receiver<KnxPacket>, mut knx: knx::Knx, _config: Arc<Config>) {

    // wait for send-requests from other threads
    loop {
        let packet = rx.recv().unwrap();
        println!("user command: {:?}", &packet);
        let command = command_from_string( packet.a );

        let (addr,frame) = match command {
            WebCommand::Dimmer { signal: Signal::EgWohnSpots, value: x } => (0x0201, knx::Command::Dimmer(x)),
            WebCommand::Dimmer { signal: Signal::EgWohnMitte, value: x } => ( 0x0202, knx::Command::Dimmer(x)),
            WebCommand::Switch { signal: Signal::EgWohnDoseFenster, value: x } => ( 0x0505, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::EgWohnDosen, value: x } => ( 0x0508, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::EgWohnDosen2, value: x } => ( 0x0504, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::EgArbeitSchreibtisch, value: x } => ( 0x0506, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::EgArbeitDosen, value: x } => ( 0x0507, knx::Command::Switch(x)),
            WebCommand::Dimmer { signal: Signal::EgArbeitSpots, value: x } => ( 0x0203, knx::Command::Dimmer(x)),
            WebCommand::Dimmer { signal: Signal::OgBadSpotsKalt, value: x } => ( 0x020f, knx::Command::Dimmer(x)),
            WebCommand::Dimmer { signal: Signal::OgBadSpotsWarm, value: x } => ( 0x0212, knx::Command::Dimmer(x)),
            WebCommand::Switch { signal: Signal::EgArbeitLight, value: x } => ( 0x0402, knx::Command::Switch(x)),
            WebCommand::RolloWert { signal: Signal::EgArbeitRollo, value: x} =>  (0x0016 /* 0/0/22 */, knx::Command::UpDownTarget(x,  200)),
            WebCommand::Dimmer { signal: Signal::EgEssenSpots, value: x } => ( 0x020A, knx::Command::Dimmer(x)),
            WebCommand::Switch { signal: Signal::EgEssenDosen, value: x } => ( 0x0504, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::EgWcLight, value: x } => ( 0x0701, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::EgKueche, value: x } => ( 0x0707, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::OgFlurSchrankzimmer, value: x } => ( 0x010a, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::OgSchlafzimmer, value: x } => ( 0x0104, knx::Command::Switch(x)),

            WebCommand::Switch { signal: Signal::Klingel, value: x } => ( 0x0600, knx::Command::Switch(x)),
            WebCommand::Switch { signal: Signal::Summer, value: x } => ( 0x0602, knx::Command::Switch(x)),

            WebCommand::Switch { signal: Signal::OgTillLight, value: x } => ( 0x0401, knx::Command::Switch(x)),

            WebCommand::Dimmer{ signal: Signal::EgFlurSpots, value: x } => ( 0x0200 + 98, knx::Command::Dimmer(x)),

            WebCommand::RolloWert { signal: Signal::EgWohnRolloEinzel, value: x} =>  (0x0010 /* 0/0/16 */, knx::Command::UpDownTarget(x,  200)),
            WebCommand::RolloWert { signal: Signal::EgWohnRolloDoppel, value: x} =>  (0x0012 /* 0/0/18 */, knx::Command::UpDownTarget(x,  200)),

            _ => { println!("command unhandled: {:?}", command); continue; }
        };

        knx.send(addr, frame);
        // match knx_ip.send( &frame ) {
        //     Ok(x) => { println!("send(): {}", x); () },
        //     Err(_) => println!("send() failed."),
        // }

    } // loop
}




fn bus_receive_thread(socket: std::net::UdpSocket, sensors: Arc<Mutex<Wetter>>) {
    let a = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/foo").expect("Could not open file");
    //    let b = std::fs::OpenOptions::new().create(true).append(true).open("/home/arbu272638/arbu-eb-rust.knx.log").expect("Could not open file");
    let b = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/foo.hex").expect("Could not open file");
//    let c = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/knx_0-2-19.log").expect("Could not open file");


    let mut logfile = std::io::BufWriter::new(a);
    let mut logfile_hex = std::io::BufWriter::new(b);
//    let mut log_0_2_19 = std::io::BufWriter::new(c);

    loop {
        let mut buf = [0_u8; 32];
        let (len, _addr) = socket.recv_from(&mut buf).expect("recv_from() failed.");
        if len < 17 {
            println!("socket datagram len too short (<19): {}", len);
            continue;
        }
        println!("Got {} bytes from {}: ", len, _addr);
        let mut hex_string = std::string::String::new();
        for a in buf[0..len].iter() {
            // print!("{:02X} ", a);
            let c = format!("{:02X} ", a);

            hex_string.push_str(&c);
        }
        // https://de.wikipedia.org/wiki/KNX-Standard
        let a_src = EibAddr(buf[10] >> 4, buf[10] & 0xf, buf[11]);
        let a_dst = EibAddr(buf[12] >> 4, buf[12] & 0xf, buf[13]);
        let _is_first = buf[9] & 0x02 != 0;
        if len < 17 {
            println!(" too short");
            continue;
        }
//	let data_len = len - 17;
        let r = Received { time: std::time::SystemTime::now(), source: a_src, dest: a_dst };
//	let mut handled = false;

        let d: Data = Data { received:r, hex_string: hex_string };
        logfile_hex.write_all(format!("{:?}\n", d).as_bytes()).expect("write_all() failed");

        logfile_hex.flush().expect("flush() failed");


        match (a_dst,len) {
            (EibAddr(0, 2, 19),2) => {
                // bewegung flur?
                println!("Schalter Flur");
            }
            (EibAddr(0, 3, 1),19) => {
                // bewegung flur?
                let value = u16::from(buf[17]) << 8 | u16::from(buf[18]);
                println!("Helligkeit Flur: {}", value);
                let val = Brightness(r.clone(), value as f32);

                let mut sensor_data = sensors.lock().expect("lock()");

                let mut line: std::string::String;
                line = format!("{:?}\n", &val);
                println!("Helligkeit 'Brightness': {:?}\n", &val);
                logfile.write_all(line.as_bytes()).expect("could not append to buffer");
                logfile.flush().expect("could not write to file.");
                sensor_data.flur_brightness = val;
            }
            (dest,19) => {
                // temperature data
                let (high, low) = (buf[17], buf[18]);

                // SEEE EBBB  BBBB BBBB
                let sign = 0x80 == high & 0x80;
                let exponent = (i32::from(high) & 0x78) >> 3;
                let base = (u16::from(high & 0x07) << 8) | u16::from(low);

                let value = match sign {
                    true => f32::from(base) * -0.01f32 * 2.0f32.powi(exponent),
                    false => f32::from(base) * 0.01f32 * 2.0f32.powi(exponent)
                };

                let mut line: std::string::String;
                line = "".to_string();

                match dest {
                    EibAddr(0, 3, 4 ) => {
                        // gruppenadresse: Temperatur Till
                        let val = Measurement::Temperature (r.clone(), value);
                        line = format!("{:?}\n", &val);
                        //		        v.b = match val { Measurement::Temperature(_, t) => t.to_string(), _ => 0.to_string() };
                        println!("Temp Till: {:?}°C", value);
                        let mut sensor_data = sensors.lock().expect("lock()");
                        sensor_data.till = val;
                    },
                    EibAddr(0, 3, 0 ) => {
                        // gruppenadresse: Temperatur Schrankzimmer
                        let val = Measurement::Temperature (r.clone(), value);
                        line = format!("{:?}\n", &val);
                        //                	v.c = match val { Measurement::Temperature(_, t) => t.to_string(), _ => 0.to_string() };
                    },
                    EibAddr(0, 3, 1) => {
                        // gruppenadresse: Helligkeit Flur EG
                        println!("Flur: {:?}", &value);
                        let mut sensor_data = sensors.lock().expect("lock()");
                        let val = Measurement::Brightness (r.clone(), value);
                        line = format!("{:?}\n", &val);
                        sensor_data.flur_brightness = val; //match val { Measurement::Brightness(_, t) => t.to_string(), _ => "".to_string() };
                    },
                    EibAddr(a, b, c) => {
                        println!("Destination address not handled (with data-len 2): {}/{}/{}",
                                 a, b, c);
                    }
                }
                logfile.write_all(line.as_bytes()).expect("could not append to buffer");
                logfile.flush().expect("could not write to file.")
            }
            (EibAddr(a, b, c),len) => {
                println!("INFO: destination address with len not handled: {}/{}/{}, len={}",
                         a, b, c, len);
            }
        };
    } // loop
}


#[tokio::main]
async fn main() {

    let base_dir = "/home/armin/git/armin-knx/".to_string();

    let config = config::read_from_file(&base_dir).expect("could not read config file");

    for r in config.file.rooms.iter() {
        println!("Raum: {:?}", r);
    }

    let h = html::create(&config).expect("html::create()");

    let mut http_data = HttpData {
        index: "".to_string(),
        html: h,
        uri_data: HashMap::new(),
    };

    http_data.uri_data.insert(
        "/img/house.png".to_string(),
        std::fs::read(base_dir.to_string() + "img/house.png").unwrap());
    http_data.uri_data.insert(
        "/img/bulb-off.png".to_string(),
        std::fs::read(base_dir.to_string() + "img/bulb-off.png").unwrap());
    http_data.uri_data.insert(
        "/img/bulb-on.png".to_string(),
        std::fs::read(base_dir.to_string() + "img/bulb-on.png").unwrap());
    http_data.uri_data.insert(
        "/img/thermometer.png".to_string(),
        std::fs::read(base_dir.to_string() + "img/thermometer.png").unwrap());

    let shared_data = Arc::new(Mutex::new(Wetter::new() ));

    let receive_socket : Option<std::net::UdpSocket>;
    if true {
        receive_socket = match std::net::UdpSocket::bind("0.0.0.0:3671") {
            Ok(s) => {
                s.join_multicast_v4(&std::net::Ipv4Addr::from_str("224.0.23.12").expect("from_str"),
                                    &std::net::Ipv4Addr::from_str("192.168.0.209").expect("from_str"))
                    .expect("join_multicast_v4()");
                s.set_multicast_loop_v4(true)
                    .expect("set_multicast_loop()");
                Some(s)
            },
            Err(e) => None, //expect("Could not bind socket")
        };
    } else {
        receive_socket = None;
    }

    let bus_data = shared_data.clone();

    let knx = knx::create();


    // channel for commands from incoming http requests to knx bus
    let (tx, rx) = channel();

    let rx_thread : Option<std::thread::JoinHandle<_>> = match receive_socket {
        Some(sock) => Some(std::thread::spawn(move || bus_receive_thread(sock, bus_data))),
        None => None
    };

    let config_for_send_thread = config.clone();
    let _s = std::thread::spawn(move || bus_send_thread(rx, knx, config_for_send_thread));


    let addr = SocketAddr::from(([0, 0, 0, 0], config.file.http.listen_port));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    use hyper::server::conn::AddrStream;
    use hyper::service::{make_service_fn, service_fn};

    // shared-ptr (r/o data)
    let http_data_arc_1 = Arc::new(http_data);

    // And a MakeService to handle each connection...
    let make_svc = make_service_fn(|socket: &AddrStream| {
        // this function is executed for each incoming connection
        let remote_addr = socket.remote_addr();
        let connection_tx = tx.clone();
        let connection_data = shared_data.clone(); //to_owned();
        let http_data_arc_2 = http_data_arc_1.clone();
        // create a service answering the requests
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let request_data = connection_data.clone();
                let request_tx = connection_tx.clone();
                let http_data_arc_3 = http_data_arc_2.clone();
                // println!("request_data: {:?}", request_data);
                async move {
                    // this function is executed for each request inside a connection
                    http_request_handler(req, http_data_arc_3, remote_addr, request_data, request_tx).await
                }
            }))
        }
    });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_svc);

    // server.
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

//     server.join().expect("Could not join() thread.")?
}



// use plotters::prelude::*;
// fn plot() -> Result<(), Box<dyn std::error::Error>> {
//     let root = BitMapBackend::new("a.png", (640, 480)).into_drawing_area();
//     root.fill(&WHITE)?;
//     let mut chart = ChartBuilder::on(&root)
//         .caption("y=x^2", ("sans-serif", 50).into_font())
//         .margin(5)
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)?;

//     chart.configure_mesh().draw()?;

//     chart
//         .draw_series(LineSeries::new(
//             (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
//             &RED,
//         ))?
//     .label("y = x^2")
//         .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

//     chart
//         .configure_series_labels()
//         .background_style(&WHITE.mix(0.8))
//         .border_style(&BLACK)
//         .draw()?;

//     Err("Nanu?".Ok())
// //     into(())
// }
