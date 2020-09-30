use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::net::{TcpStream, SocketAddr};
use std::io::{Write, BufRead};

use std::string::String;
use std::convert::Infallible;
use hyper::{Body, Request, Response, Server};


//use std::sync::mpsc::channel()
// for read_into_string()
use std::io::prelude::*;

#[derive(Debug)]
struct Wetter {
    a: String,
    b: String,
    c: String
}


async fn _hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {

    let f = std::fs::File::open("/tmp/foo");
    if f.is_err() {
	return Ok(Response::builder().status(400).body("ERROR 0".into() ).unwrap());
    }

    let mut data = std::string::String::new();
    if f.unwrap().read_to_string(&mut data).is_err() {
	return Ok(Response::builder().status(300).body("ERROR 1".into() ).unwrap());
    }

    let mut handlebars = handlebars::Handlebars::new();
    if handlebars.register_template_file("/", "template/index.html").is_err() {
	return Ok(Response::builder().status(300).body("ERROR 2".into() ).unwrap());
    }

    #[derive(serde::Serialize)]
    struct Info {
	foo: std::string::String,
	bar: i64,
    };

    let info = Info { foo: "foo".to_owned(),
		      bar: 1231,
    };

    let output = handlebars.render("/", &info);

    Ok(Response::builder().status(200).body( output.unwrap().into() ).unwrap())
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
struct EibAddr (u8, u8, u8);

fn bus_thread(u: std::net::UdpSocket, data: Arc<Mutex<Wetter>>) {
    let a = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/foo").expect("Could not open file");


    let mut logfile = std::io::BufWriter::new(a);

    loop {
        let mut buf = [0_u8;32];
        let (len,addr) = u.recv_from(&mut buf).expect("recv_from() failed.");
        print!("Got {} bytes from {}: ", len, addr);
        for a in buf[0..len].iter() {
            print!("{:02X} ", a);
        }
        if len < 11 {
            println!(" too short.");
            continue;
        }
        // https://de.wikipedia.org/wiki/KNX-Standard
        let a_src = EibAddr(buf[8] >> 4, buf[8] & 0xf, buf[9]);
        let a_dst = EibAddr(buf[10] >> 4, buf[10] & 0xf, buf[11]);
        let is_first = buf[9] & 0x02 != 0;
        // 0000 0010b  0x02
        // 0000 0101b  0x05
        // 0000 1101b  0x0d
        println!("  -- {:?}->{:?} (first: {})", a_src, a_dst, is_first);
        if len == 15 {
            // switch
            let onoff = buf[14] & 0x1 == 1;
            println!("On-Off: {}", onoff);
        }
        if len == 17 {
            // temperature data
            let (high,low) = (buf[15], buf[16]);
            // SEEE EBBB  BBBB BBBB
            let sign = 0x80 == high & 0x80;
            let exponent = (i32::from(high) & 0x78) >> 3;
            let base = (u16::from(high & 0x07) << 8) | u16::from(low);
            println!("high,low: {:02X}, {:02X}", high, low);
            println!("sign: {}, exponent: {}, base: {}", sign, exponent, base);
            let value = match sign {
                true =>  f32::from(base) * -0.01f32 * 2.0f32 .powi (exponent),
                false => f32::from(base) *  0.01f32 * 2.0f32 .powi(exponent)
            };

            #[derive(Debug)]
            struct Measurement { time: std::time::SystemTime, value: f32 };

            let val = Measurement { time: std::time::SystemTime::now(), value: value };
	        let mut _v = data.lock().unwrap();
            println!("{:?}\n", val);
            let line = format!("{:?}\n", val);
            logfile.write_all(line.as_bytes()).expect("could not append to buffer");
            logfile.flush().expect("could not write to file.");
        }
    }
}


#[tokio::main]
async fn main() {
    // let addr = std::net::SocketAddrV4::from_str("0.0.0.0:1234").unwrap();
    let shared_data = Arc::new(Mutex::new(Wetter {
	a: "Ah".to_owned(),
	b: "Beh".to_owned(),
	c: "Zeh".to_owned()
    }));


    let u = std::net::UdpSocket::bind("0.0.0.0:51000").expect("Could not bind socket");
    u.join_multicast_v4(
        &std::net::Ipv4Addr::from_str("239.192.39.238").unwrap(),
        &std::net::Ipv4Addr::from_str("192.168.0.8").unwrap()).expect("");

    let _j = std::thread::spawn(move || bus_thread(u, shared_data));


    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    use hyper::server::conn::AddrStream;
    use hyper::service::{make_service_fn, service_fn};
    //let _service_myfunc = | _req: Request<Body> | {
    //    println!("ASDASD");
    //};
    let service = service_fn(|req: Request<Body> | async move {
        if req.version() == hyper::http::version::Version::HTTP_11 {
            // let a = std::sync::Arc::clone(&shared_data);
            // println!("Wetter: {:?}", &a); //foo.deref().lock().unwrap());
            Ok(Response::new(Body::from("Hello World")))
        } else {
            // note: better: return a response with status code
            Err("not HTTP/1.1, abort connection")
        }
    });
    let make_svc = make_service_fn(|_socket: &AddrStream| async move {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service)
    });
    let server = Server::bind(&addr).serve(make_svc);


    // We'll bind to 127.0.0.1:3001
    // let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    // let l = std::net::TcpListener::bind(addr).unwrap();
    // loop {
    // 	let x = l.accept();
    //     let  (stream, addr) = x.unwrap();
    //     let h = std::thread::spawn(move || mythread(stream, addr));
    //     h.join().expect("Could not join() thread.");
    // }

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    // j.join().expect("Could not join() thread.")
}
