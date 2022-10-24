use std::net::{Ipv4Addr, SocketAddr};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc};
use regex;
use crate::config::Config;
use crate::data;
use crate::data::{Dimension, Measurement};

use tokio::net::UdpSocket;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EibAddr(u8, u8, u8);

impl EibAddr {
    pub fn to_string(&self) -> String {
        format!("{}/{}/{}", self.0, self.1, self.2).to_string()
    }
}

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


pub enum Command {
    Dimmer(u8), //percent
    Switch(bool),
    UpDownTarget(u8,u8), //value,max
}


pub struct Knx {
    socket: Option<UdpSocket>,
    config: Arc<Config>,
}

#[derive(Debug, Copy, Clone)]
pub struct Message {
    src: EibAddr,
    dst: EibAddr,
    sensor: Option<Measurement>,
}

impl Message {
    pub fn from_raw(raw: &[u8]) -> Message {
        let src = EibAddr(raw[10] >> 4, raw[10] & 0xf, raw[11]);
        let dst = EibAddr(raw[12] >> 4, raw[12] & 0xf, raw[13]);

        Message { src, dst, sensor: None }
    }
}


pub fn create(config: Arc<Config>) -> Result<Knx, String> {

    // to send packets:
    // socket.connect("224.0.23.12:3671")
    //     .map_err(|e|e.to_string())?;

    let knx = Knx { socket: None, config };
    Ok(knx)
}



impl Knx {
    pub async fn thread_function(&mut self) {
        let bind_addr = "0.0.0.0:3671"; // 3671
        let socket = UdpSocket::bind(bind_addr)
            .await.map_err(|e|e.to_string()).expect("bind()");

        socket.join_multicast_v4(self.config.knx_multicast_group,
                                 self.config.knx_multicast_interface)
            .map_err(|e|e.to_string()).expect("join multicast v4");

        self.socket = Some(socket);
        loop {
            println!("knx: loop begin");
            let mut buf = [0; 128];
            println!("waiting for frame...");
            let socket = self.socket.as_ref().unwrap();
            let (number_of_bytes, addr) = socket.recv_from(&mut buf)
                .await.expect("can not call recv_from() on udp soecket");
            // cleate a slice
            let filled_buf = &mut buf[..number_of_bytes];
            println!("message from {}: {:?}", addr, &filled_buf);

            let len = filled_buf.len();
            if len < 17 {
                println!("message with size {} is too short", len);
                continue;
            }
            if len > 50 {
                println!("message with size {} is too long", len);
                continue;
            }

            let msg = Message::from_raw(filled_buf);

            println!("message from {:?}", msg.src.to_string());
        }
    }


    // pub fn sendRaw(&mut self, msg: Message) {
    // 	self.socket.send(msg.raw.as_slice()).expect("send() failed");
    // }

    pub fn send(&mut self, grp: u16, command: Command) {
        let msg = match command {
            Command::Dimmer(x) => create_knx_frame_dimmer(grp, x),
            Command::Switch(x) => create_knx_frame_onoff(grp, x),
            Command::UpDownTarget(x,max) => create_knx_frame_rollo(grp, x, max)
        };

       //  self.socket.send(msg.raw.as_slice()).expect("send() failed");
    }

    pub fn handle_knx_frame(&mut self, msg: &Message) {

    }
}

pub fn create_knx_frame_onoff(grp: u16, onoff: bool) -> Message {

    let mut dst = vec![ (grp >> 8) as u8, (grp & 0xff) as u8 ];
    let mut v = vec![
        // knx/ip header
        // HEADER_LEN: 0x06,  VERSION: 0x10 (1.0), ROUTING_INDIXATION (0x05, 0x03), TOTAL-LEN(0x00, 0x11)
        0x06u8, 0x10, 0x05, 0x30, 0x00, 0x11,

        0x29, // data indication
        0x00, // extra-info
        0xbc, //low-prio,
        0xe0, // to-group-address (1 << 7) | hop-count-6 (6 << 5) | extended-frame-format (0x0)
        0x12, 0x7e  // src: 0x127e -> 1.2.126
    ];
    v.append( &mut dst );
    v.push( 1u8 ); //len
    v.push( 0x00 ); // 'TPCI'
    if onoff { v.push( 0x81u8 ); } else { v.push( 0x80 ); }
    println!(" onoff: {:X?}", v);

    Message { src: EibAddr(0, 0, 0), dst: EibAddr(0, 0, 0),
    sensor: None}
}


pub fn create_knx_frame_dimmer(grp: u16, percent: u8) -> Message
{
    let mut dst = vec![ (grp >> 8) as u8, (grp & 0xff) as u8 ];
    let mut v = vec![
        // knx/ip header
        // HEADER_LEN: 0x06,  VERSION: 0x10 (1.0), ROUTING_INDIXATION (0x05, 0x03), TOTAL-LEN(0x00, 0x11)
        0x06u8, 0x10, 0x05, 0x30, 0x00, 0x12,

        0x29, // data indication
        0x00, // extra-info
        0xbc, //low-prio,
        0xe0, // to-group-address (1 << 7) | hop-count-6 (6 << 5) | extended-frame-format (0x0)
        0x12, 0x7e  // src: 0x127e -> 1.2.126
    ];
    v.append( &mut dst );
    v.push( 2u8 ); //len
    v.push( 0x00 ); // 'TPCI'
    v.push( 0x80 );
    v.push( percent  );

    println!(" helligkeitswert: {:X?}", v);


    Message { src: EibAddr(0, 0, 0), dst: EibAddr(0, 0, 0),
        sensor: None}
    // Message { raw: v }
}



// 'full': entspricht wert, der fuer vollstaendiges schliessen gesendet werden muss, z.B. 205
fn create_knx_frame_rollo(grp: u16, percent: u8, full: u8) -> Message
{
    let t = percent as u16 * full as u16 / 100;
    let tx_raw = if t > 255 { 255 } else { t as u8 };
    let mut dst = vec![ (grp >> 8) as u8, (grp & 0xff) as u8 ];
    let mut v = vec![
        // knx/ip header
        // HEADER_LEN: 0x06,  VERSION: 0x10 (1.0), ROUTING_INDIXATION (0x05, 0x03), TOTAL-LEN(0x00, 0x11)
        0x06u8, 0x10, 0x05, 0x30, 0x00, 0x12,

        0x29, // data indication (rollo: 0x2e)
        0x00, // extra-info
        0xbc, //low-prio,
        0xe0, // to-group-address (1 << 7) | hop-count-6 (6 << 5) | extended-frame-format (0x0)
        0x12, 0x7e  // src: 0x127e -> 1.2.126
    ];
    v.append( &mut dst );
    v.push( 2u8 ); //len
    v.push( 0x00 ); // 'TPCI'
    v.push( 0x80 );
    v.push( tx_raw  );

    println!(" rollo-wert: {:X?}", v);


    Message { src: EibAddr(0, 0, 0), dst: EibAddr(0, 0, 0),
        sensor: None}
}

