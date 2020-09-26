
use std::str::FromStr;
use std::net::{TcpStream, SocketAddr};
use std::io::{Write, BufRead};


fn mythread(mut stream: TcpStream, _addr: SocketAddr) {
    eprintln!("New connection: {:?}", stream);
    // sleep(std::time::Duration::from_secs(1));
    stream.set_read_timeout(Option::from(std::time::Duration::from_secs(5))).expect("could not set read timeout");
    let a= std::string::String::from("ASD\n");

    stream.write_all(a.as_bytes()).expect("write failed");
    stream.write_all("Hallo\n\n".as_bytes()).expect("write failed");

    let r = std::io::BufReader::new(&stream);

    for l in r.lines() {
        println!("Zeile: {}", l.unwrap())
    }
}

#[derive(Debug)]
struct EibAddr (u8, u8, u8);

fn bus_thread(u: std::net::UdpSocket) {
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
        let a_src = EibAddr(buf[0] >> 4, buf[0] & 0xf, buf[1]);
        let a_dst = EibAddr(buf[2] >> 4, buf[2] & 0xf, buf[3]);
        println!("  -- {:?}->{:?}", a_src, a_dst);
        // temperature data
        if len == 17 {
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
            println!("{:?}\n", val);
            let line = format!("{:?}\n", val);
            logfile.write_all(line.as_bytes()).expect("could not append to buffer");
            logfile.flush().expect("could not write to file.");
        }
    }
}


fn main() {
    let addr = std::net::SocketAddrV4::from_str("0.0.0.0:1234").unwrap();

    let u = std::net::UdpSocket::bind("0.0.0.0:51000").expect("Could not bind socket");
    u.join_multicast_v4(
        &std::net::Ipv4Addr::from_str("239.192.39.238").unwrap(),
        &std::net::Ipv4Addr::from_str("192.168.0.78").unwrap()).expect("");

    let j = std::thread::spawn(move || bus_thread(u));

    let l = std::net::TcpListener::bind(addr).unwrap();
    while let x = l.accept() {
        let  (stream, addr) = x.unwrap();
        let h = std::thread::spawn(move || mythread(stream, addr));
        h.join().expect("Could not join() thread.");
    }

    j.join().expect("Could not join() thread.")
}
