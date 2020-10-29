use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::net::{TcpStream, SocketAddr};
use std::io::{Write, BufRead};

use std::string::String;
use std::convert::Infallible;
use hyper::body;
use hyper::{Body, Request, Response, Server};

//use std::sync::mpsc::channel()
// for read_into_string()

#[derive(Debug, Copy, Clone)]
struct Received { time: std::time::SystemTime, source: EibAddr, dest: EibAddr }

#[derive(Debug)]
struct Measurement { received: Received, value: f32 }

#[derive(Debug)]
struct DimmerZielwert { received: Received, value: u8 }

#[derive(Debug)]
struct Data { received: Received, hex_string: String }

#[derive(Debug, Copy, Clone)]
struct EibAddr(u8, u8, u8);

#[derive(Debug)]
struct Wetter {
    a: String,
    b: String,
    c: String,
}


async fn hello_world(req: Request<Body>, remote_addr: SocketAddr, wetter: Arc<Mutex<Wetter>>) -> Result<Response<Body>, Infallible> {
    // let f = std::fs::File::open("/tmp/foo");
    // if f.is_err() {
    //     return Ok(Response::builder().status(400).body("ERROR 0".into()).unwrap());
    // }
    //
    // let mut data = std::string::String::new();
    // if f.unwrap().read_to_string(&mut data).is_err() {
    //     return Ok(Response::builder().status(300).body("ERROR 1".into()).unwrap());
    // }
    println!("METHOD: {:?}", &req.method());
    let bytes = body::to_bytes(req.into_body()).await.expect("failed!");
    let body_str = String::from_utf8(bytes.to_vec()).expect("nody was not valid utf8");

    //req.body().on_upgrade().await.unwrap().read_to_string().await.unwrap();

    println!("BODY: {:?}",body_str);

    let mut handlebars = handlebars::Handlebars::new();
    if handlebars.register_template_file("/", "template/index.html").is_err() {
        return Ok(Response::builder().status(300).body("ERROR 2".into()).unwrap());
    }

    let mut _w = wetter.lock().unwrap();
    // wetter.lock()a.push('.');

    _w.a.push('.');

    #[derive(serde::Serialize)]
    struct Info {
        foo: std::string::String,
        bar: i64,
        addr: String,
        wetter_a: String,
    }

    let info = Info {
        foo: "foo".to_owned(),
        bar: 1231,
        addr: format!("{:?}", remote_addr.to_string()),
        wetter_a: _w.a.clone(),
    };

    let output = handlebars.render("/", &info);

    Ok(Response::builder().status(200).body(output.unwrap().into()).unwrap())
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

use std::sync::mpsc::channel; //function
fn bus_send_thread(rx: std::sync::mpsc::Receiver<KnxPacket>) {
    loop {
	let packet = rx.recv().unwrap();
	println!("Packet: {:?}", packet);
    }
}


fn bus_thread(u: std::net::UdpSocket, data: Arc<Mutex<Wetter>>) {
    let a = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/foo").expect("Could not open file");
//    let b = std::fs::OpenOptions::new().create(true).append(true).open("/home/arbu272638/arbu-eb-rust.knx.log").expect("Could not open file");
    let b = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/foo.hex").expect("Could not open file");


    let mut logfile = std::io::BufWriter::new(a);
    let mut logfile_hex = std::io::BufWriter::new(b);

    loop {
        let mut buf = [0_u8; 32];
        let (len, addr) = u.recv_from(&mut buf).expect("recv_from() failed.");
        print!("Got {} bytes from {}: ", len, addr);
        let mut hex_string = std::string::String::new();
        for a in buf[0..len].iter() {
            // print!("{:02X} ", a);
            let c = format!("{:02X} ", a);

            hex_string.push_str(&c);
        }
        // https://de.wikipedia.org/wiki/KNX-Standard
        let a_src = EibAddr(buf[8] >> 4, buf[8] & 0xf, buf[9]);
        let a_dst = EibAddr(buf[10] >> 4, buf[10] & 0xf, buf[11]);
        let _is_first = buf[9] & 0x02 != 0;
        let r = Received { time: std::time::SystemTime::now(), source: a_src, dest: a_dst };

        let d: Data = Data { received: r, hex_string: hex_string };
        logfile_hex.write_all(format!("{:?}\n", d).as_bytes()).expect("write_all() failed");
        logfile_hex.flush().expect("flush() failed");
        if len < 11 {
            println!(" too short.");
            continue;
        }

        // 0000 0010b  0x02
        // 0000 0101b  0x05
        // 0000 1101b  0x0d
        // println!("{}  -- {:?}->{:?} (first: {})", &hex_string, a_src, a_dst, is_first);
        if len == 15 {
            // switch
            let onoff = buf[14] & 0x1 == 1;
            println!("On-Off: {}", onoff);
        }
        if len == 16 {
            // Wert 0..255 unsigned (z.B. Zielwert Dimmer setzen)
            let _value = f32::from(buf[15]);
        }
        if len == 17 {
            // temperature data
            let (high, low) = (buf[15], buf[16]);

            // SEEE EBBB  BBBB BBBB
            let sign = 0x80 == high & 0x80;
            let exponent = (i32::from(high) & 0x78) >> 3;
            let base = (u16::from(high & 0x07) << 8) | u16::from(low);

            let value = match sign {
                true => f32::from(base) * -0.01f32 * 2.0f32.powi(exponent),
                false => f32::from(base) * 0.01f32 * 2.0f32.powi(exponent)
            };


            let val = Measurement { received: r, value: value };
            let mut _v = data.lock().unwrap();
            _v.a = val.value.to_string();
            println!("{:?}\n", val);
            let line = format!("{:?}\n", val);
            logfile.write_all(line.as_bytes()).expect("could not append to buffer");
            logfile.flush().expect("could not write to file.")
        }
    }
}


#[tokio::main]
async fn main() {
    // let addr = std::net::SocketAddrV4::from_str("0.0.0.0:1234").unwrap();
    let shared_data = Arc::new(Mutex::new(Wetter {
        a: "Ah".to_owned(),
        b: "Beh".to_owned(),
        c: "Zeh".to_owned(),
    }));

    let (tx, rx) = channel();
    let _s = std::thread::spawn(move || bus_send_thread(rx));


    let u = std::net::UdpSocket::bind("0.0.0.0:51000").expect("Could not bind socket");
    u.join_multicast_v4(
        &std::net::Ipv4Addr::from_str("239.192.39.238").unwrap(),
        &std::net::Ipv4Addr::from_str("192.168.0.208").unwrap()).expect("");

    let bus_data = shared_data.clone();
    let _j = std::thread::spawn(move || bus_thread(u, bus_data));


    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    use hyper::server::conn::AddrStream;
    use hyper::service::{make_service_fn, service_fn};

    // And a MakeService to handle each connection...
    let make_svc = make_service_fn(|socket: &AddrStream| {
        // this function is executed for each incoming connection
        let remote_addr = socket.remote_addr();

        let connection_data = shared_data.to_owned();
        // create a service answering the requests
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let _request_data = connection_data.clone();
                println!("_request_data: {:?}", _request_data);
                async move {
                    // this function is executed for each request inside a connection
                    let _a = _request_data.clone();
                    hello_world(req, remote_addr, _a).await
                }
            }))
        }
    });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    // j.join().expect("Could not join() thread.")
}
