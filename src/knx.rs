use std::borrow::Borrow;
use std::error::Error;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use regex;
use tokio::io;
use crate::config::{Config, EibAddr, Sensor};
use crate::data::{Dimension, Measurement, Unit};

use tokio::net::UdpSocket;
use crate::data;
use crate::data::Dimension::{Brightness, Temperature};
use crate::data::Unit::{Celsius, Lux};


fn parse_addr(s: &str) -> Result<EibAddr,String> {
    // haupt/mittel/untergruppe
    let re = regex::Regex::new(r"^(?P<haupt>[[:digit:]]+)/(?P<mittel>[[:digit:]]+)/(?P<unter>[[:digit:]]+)$").unwrap();
    match re.captures(s) {
        Some(cap) => {
            let a = EibAddr(
                cap["haupt"].parse::<u8>().unwrap(),
                cap["mittel"].parse::<u8>().unwrap(),
                cap["unter"].parse::<u8>().unwrap());

            Ok(a)
        },
        None => {
            let message = String::new() + "string '" + s + "' does not match format 'x/y/z'";
            Err(message)
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Dimmer(u8), //percent
    Switch(bool),
    UpDownTarget(u8,u8), //value,max
}


pub struct Knx {
    config: Arc<Config>,
    data: Arc<Mutex<data::Data>>,
}


#[derive(Debug, Copy, Clone)]
pub struct Message {
    src: EibAddr,
    dst: EibAddr,
    measurement: Option<Measurement>,
}

#[derive(Debug, Clone)]
pub struct SendMessage {
    raw: Vec<u8>,
}

#[derive(Debug)]
pub struct KnxSocket {
    udp: UdpSocket,
}

impl KnxSocket {
    // do not bind to local address
    pub async fn create() -> tokio::io::Result<KnxSocket> {
        let udp = UdpSocket::bind( ("127.0.0.1", 8090)  ).await?;
        Ok ( KnxSocket{ udp } )
    }
    pub async fn bind(local_address: &String, local_port: u16) -> tokio::io::Result<KnxSocket> {
        let udp = UdpSocket::bind( (local_address.as_str(), local_port)).await?;
        Ok ( KnxSocket{ udp } )
    }
    pub async fn join(&mut self, multicast_group: String, multicast_interface: String) -> io::Result<()> {
        self.udp
            .join_multicast_v4(multicast_group.parse().unwrap(), multicast_interface.parse().unwrap())?;
        Ok( () )
    }
    pub async fn send(&mut self, grp: u16, command: &Command) {
        let msg = match command {
            //Command::Dimmer(x) => create_knx_frame_dimmer(grp, x),
            Command::Switch(x) => create_knx_frame_onoff(grp, *x),
            //Command::UpDownTarget(x,max) => create_knx_frame_rollo(grp, x, max),
            _ => Err( () )
        };

        let msg = msg.expect("could not create knx frame for command {command:?}");

        self.udp.send(msg.raw.as_slice()).await.expect("send() failed");
    }
}

impl Message {
    pub fn from_raw(raw: &[u8]) -> Message {
        let src = EibAddr(raw[10] >> 4, raw[10] & 0xf, raw[11]);
        let dst = EibAddr(raw[12] >> 4, raw[12] & 0xf, raw[13]);

        let command: Option<Command> = None;
        let mut measurement: Option<Measurement> = None;

        match raw.len() {
            19 => {
                let value = convert_16bit_float(raw[17], raw[18]);
               //  println!("Value (raw[17]|raw[18]): {}", val);
                let m = Measurement { dimension: Dimension::Brightness, unit: Unit::One, value: Some(value) };
                measurement = Some(m);
            },
            17 => {
                println!("len=17: on/off value?");
            },
            len => {
                println!("len={}: ???", len);
            }
        }

        Message { src, dst, measurement }
    }
}


pub fn create(config: Arc<Config>, data: Arc<Mutex<data::Data>>) -> Result<Knx, String> {
    let knx = Knx { config, data };
    Ok(knx)
}


impl Knx {
    pub async fn thread_function(&mut self) -> Result<(),()> {
        let bind_addr = (
            self.config.knx_multicast_interface,
            self.config.knx_multicast_port);
        let mut socket = UdpSocket::bind(bind_addr)
            .await
            .expect("could not bind socket");
            // await.map_err(|e|e.to_string()).expect("bind()");
        socket
            .join_multicast_v4(
                self.config.knx_multicast_group,
                self.config.knx_multicast_interface)
            .expect("Could not join multicast group");

        loop {
            println!("knx: loop begin");
            let mut buf = [0; 128];
            println!("waiting for frame...");
            let (number_of_bytes, addr) = socket.recv_from(&mut buf)
                .await.expect("can not call recv_from() on udp socket");
            // create a slice
            let filled_buf = &mut buf[..number_of_bytes];
            println!(
                "message {:02X?}", &filled_buf);
            for a in 0..number_of_bytes {
                print!("{:2}  ", a); };

            match filled_buf.len() {
                x if x < 16 => {
                    println!("message with size {} is too short", x);
                    continue;
                },
                x if x > 50 => {
                    println!("message with size {} is too long", x);
                    continue;
                },
                any => {}
            };

            let msg = Message::from_raw(filled_buf);
            let a = self.get_sensor_from(&msg.src);
            if a.is_some() {
                let (id, sensor) = a.unwrap();
                println!("message from sensor {id}");
                let mut data = self.data.lock().unwrap();
                match data.get_mut(id) {
                    Some(mut m) => m.value = msg.measurement.expect("no measurement in frame").value,
                    None => eprintln!("sensor not known in data struct?")
                }
            } else {
                eprintln!("No sensor known with eib addr {:?}", &msg.src);
            }

            println!("knx-message from {:?}: measurement={:?}", msg.src, msg.measurement);
        }
    }


    pub fn get_sensor_from(&self, addr: &EibAddr) -> Option<(&String, &Sensor)> {
        let addr_string = format!("{}/{}/{}", addr.0, addr.1, addr.2);
        for (id, sensor) in &self.config.sensors {
            if sensor.eibaddr == addr_string {
                return Some( (id, sensor) );
            }
        }
        None
    }

}

pub fn create_knx_frame_onoff(grp: u16, onoff: bool) -> Result<SendMessage, ()> {

    let mut dst = vec![ (grp >> 8) as u8, (grp & 0xff) as u8 ];
    let mut raw = vec![
        // knx/ip header
        // HEADER_LEN: 0x06,  VERSION: 0x10 (1.0), ROUTING_INDIXATION (0x05, 0x03), TOTAL-LEN(0x00, 0x11)
        0x06u8, 0x10, 0x05, 0x30, 0x00, 0x11,

        0x29, // data indication
        0x00, // extra-info
        0xbc, //low-prio,
        0xe0, // to-group-address (1 << 7) | hop-count-6 (6 << 5) | extended-frame-format (0x0)
        0x12, 0x7e  // src: 0x127e -> 1.2.126
    ];
    raw.append( &mut dst );
    raw.push( 1u8 ); //len
    raw.push( 0x00 ); // 'TPCI'
    if onoff { raw.push( 0x81u8 ); } else { raw.push( 0x80 ); }
    println!(" onoff: {:X?}", raw);

    Ok ( SendMessage { raw } )
}

//
// pub fn create_knx_frame_dimmer(grp: u16, percent: u8) -> Message
// {
//     let mut dst = vec![ (grp >> 8) as u8, (grp & 0xff) as u8 ];
//     let mut v = vec![
//         // knx/ip header
//         // HEADER_LEN: 0x06,  VERSION: 0x10 (1.0), ROUTING_INDIXATION (0x05, 0x03), TOTAL-LEN(0x00, 0x11)
//         0x06u8, 0x10, 0x05, 0x30, 0x00, 0x12,
//
//         0x29, // data indication
//         0x00, // extra-info
//         0xbc, //low-prio,
//         0xe0, // to-group-address (1 << 7) | hop-count-6 (6 << 5) | extended-frame-format (0x0)
//         0x12, 0x7e  // src: 0x127e -> 1.2.126
//     ];
//     v.append( &mut dst );
//     v.push( 2u8 ); //len
//     v.push( 0x00 ); // 'TPCI'
//     v.push( 0x80 );
//     v.push( percent  );
//
//     println!(" helligkeitswert: {:X?}", v);
//
//
//     Message { src: EibAddr(0, 0, 0), dst: EibAddr(0, 0, 0),
//         measurement: None}
//     // Message { raw: v }
// }
//

//
// // 'full': entspricht wert, der fuer vollstaendiges schliessen gesendet werden muss, z.B. 205
// fn create_knx_frame_rollo(grp: u16, percent: u8, full: u8) -> Message
// {
//     let t = percent as u16 * full as u16 / 100;
//     let tx_raw = if t > 255 { 255 } else { t as u8 };
//     let mut dst = vec![ (grp >> 8) as u8, (grp & 0xff) as u8 ];
//     let mut v = vec![
//         // knx/ip header
//         // HEADER_LEN: 0x06,  VERSION: 0x10 (1.0), ROUTING_INDIXATION (0x05, 0x03), TOTAL-LEN(0x00, 0x11)
//         0x06u8, 0x10, 0x05, 0x30, 0x00, 0x12,
//
//         0x29, // data indication (rollo: 0x2e)
//         0x00, // extra-info
//         0xbc, //low-prio,
//         0xe0, // to-group-address (1 << 7) | hop-count-6 (6 << 5) | extended-frame-format (0x0)
//         0x12, 0x7e  // src: 0x127e -> 1.2.126
//     ];
//     v.append( &mut dst );
//     v.push( 2u8 ); //len
//     v.push( 0x00 ); // 'TPCI'
//     v.push( 0x80 );
//     v.push( tx_raw  );
//
//     println!(" rollo-wert: {:X?}", v);
//
//
//     Message { src: EibAddr(0, 0, 0), dst: EibAddr(0, 0, 0),
//         measurement: None}
// }

// slice: &[u8], array: &[u8; 2]
pub fn convert_16bit_float(high:u8, low:u8) -> f32 {
    // MEEEMMMM MMMMMMMM
    let mantissa = ((high & 0x07) as u16 * 256) + low as u16;
    let negative = high & 0x80 == 0x80;
    let exponent = match (high & 0x70) { 0 => 1, other => other as i32 };
    println!("{} {mantissa} .powi ( {exponent} )", (if negative { "-" } else { "" } ));

    let mut out = (mantissa as f32).powi(exponent);
    if negative { out *= -1f32 };
    println!("--> {:02X}:{:02X} = {}", high, low, out);
    out * 0.01 // resolution: 0.01
}