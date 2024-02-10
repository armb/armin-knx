use std::borrow::Borrow;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::net::{Ipv4Addr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use regex;
use tokio::io;
use crate::config::{Config, EibAddr, Sensor};
use crate::data::{Dimension, Unit, Measurement};

use tokio::net::UdpSocket;
use crate::data;

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
    Shutter(u8),
}


pub struct Knx {
    config: Arc<Config>,
    data: Arc<Mutex<data::Data>>,
    log: Mutex<File>,
}


#[derive(Debug, Clone)]
pub struct Message {
    timestamp: SystemTime,
    src: EibAddr,
    dst: EibAddr,
    measurement: Option<Measurement>,
    raw: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SendMessage {
    raw: Vec<u8>,
}

#[derive(Debug)]
pub struct KnxSocket {
    udp: std::net::UdpSocket,
}


impl KnxSocket {
    // do not bind to local address
    pub fn create() -> io::Result<KnxSocket> {
        let udp = std::net::UdpSocket::bind( ("0.0.0.0", 8090)  )?;
        Ok ( KnxSocket{ udp } )
    }
    pub fn bind(local_address: &String, local_port: u16) -> io::Result<KnxSocket> {
        let udp = std::net::UdpSocket::bind( (local_address.as_str(), local_port))?;
        Ok ( KnxSocket{ udp } )
    }
    pub fn join(&mut self, multicast_group: String, multicast_interface: String) -> io::Result<()> {
        let group = Ipv4Addr::from_str(&*multicast_group)
            .expect("should be valid multicast-group string");
        let interface = Ipv4Addr::from_str(&*multicast_interface)
            .expect("should be valid interface-address string");
        self.udp.join_multicast_v4(&group, &interface)
    }
    pub fn connect(&mut self, address: &String) -> io::Result<()> {
        self.udp.connect(address)
    }

    pub fn send(&mut self, group_string: &String, command: &Command) -> Result<(), String> {
        let addr = parse_addr(group_string.as_str()).expect("group string invalid");
        let a = (addr.0 as u16) << 12;
        let b = (addr.1 as u16) << 8;
        let c = (addr.2 as u16) ;
        let grp: u16 = a | b | c;

        let msg = match command {
            Command::Dimmer(x) => create_knx_frame_dimmer(grp, x.clone()),
            Command::Switch(x) => create_knx_frame_onoff(grp, x.clone()),
            Command::Shutter(x) => create_knx_frame_rollo(grp, x),
        }.expect("knx frame not defined for command");

        println!("SEND: {group_string} -> {addr:?} -> {grp}: {msg:0X?}");

        self.udp.send(msg.raw.as_slice())
            .expect("send() failed");

        Ok( () )
    }
}

impl Message {
    pub fn from_raw(raw: Vec<u8>, timestamp: SystemTime) -> Result<Message, ()> {
        if raw.len() < 14 {
            return Err(());
        }
        let src = EibAddr(raw[10] >> 4, raw[10] & 0xf, raw[11]);
        let dst = EibAddr(raw[12] >> 4, raw[12] & 0xf, raw[13]);

        let _command: Option<Command> = None;
        let mut measurement: Option<Measurement> = None;

        match raw.len() {
            19 => {
                //let value = convert_16bit_float(raw[17], raw[18]);
                // mantissa: 11 bits (1/100).
                let m: i32 = ((0x07 & raw[17] as i32) << 8) + raw[18] as i32;
                let e: i32 = (raw[17] >> 3  & 0x0f) as i32;
                let is_negative = raw[17] & 0x70 == 0x70;
                // val = m * 2^e
                let value = if is_negative {
                    -0.01 * (m << e) as f32
                } else {
                    0.01 * (m << e) as f32
                };

                let (dimension, unit) = match raw[13] {
                    0x01 => (Dimension::Brightness, Unit::Lux),
                    0x03 => (Dimension::Brightness, Unit::Lux),
                    0x04 => (Dimension::Temperature, Unit::Celsius),
                    0x06 => (Dimension::Brightness, Unit::Lux),
                    _ => (Dimension::None, Unit::One)
                };

                // println!("message={:02X?}", raw);
                // println!("float-Value (raw[17]|raw[18]): negative={is_negative}, e={e}, m={m} --> {}", val);
                measurement = Some( Measurement{timestamp, dimension, unit, value: Some(value) });
            },
            17 => {
                let onoff = if raw[16] == 0x80 { 0.0 } else { 1.0 };
                measurement = Some( Measurement{timestamp, dimension: Dimension::OnOff, unit: Unit::One, value: Some(onoff)});
                // println!("len=17: on/off value?");
            },
            18 => {
                let percent = raw[17] as f32 * 100. / 255.;
                // println!("len=18, percent={percent}");
                measurement = Some( Measurement{timestamp, dimension: Dimension::Percent, unit: Unit::One, value: Some(percent)});

            }
            len => {
                println!("len={}: ???", len);
            }
        }

        Ok(Message { timestamp, src, dst, measurement, raw: raw.to_vec() })
    }
}


pub fn create(config: Arc<Config>, data: Arc<Mutex<data::Data>>) -> Result<Knx, String> {

    let path = "log.txt";
    let mut log = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path).expect("Could not open log.txt");
    log.seek(SeekFrom::End(0)).expect("seek() failed");
    let text = "-------\n".as_bytes();
    log.write(text).expect("write() failed");
    let knx = Knx { config, data, log: Mutex::new(log) };
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
            // println!("knx: loop begin");
            let mut buf = [0; 128];
            // println!("waiting for frame...");
            let (number_of_bytes, _addr) = socket.recv_from(&mut buf)
                .await.expect("can not call recv_from() on udp socket");
            // create a slice
            let filled_buf = &mut buf[..number_of_bytes];
            // println!(
            //     "message {:02X?}", &filled_buf);
            let timestamp = SystemTime::now();

            match Message::from_raw(filled_buf.to_vec(), timestamp) {
                Ok(msg) => {
                    //println!("knx-message from {:?}: measurement={:?}, raw={:02X?}", msg.src, msg.value, &filled_buf);
                    if let Some( (id, _sensor) ) = self.get_sensor_from(&msg.dst) {
                        let value_string = match msg.measurement {
                            Some(m) => match m.value {
                                Some(v) => v.to_string(),
                                None => "?".to_string()
                            },
                            None => "?".to_string()
                        };
                        println!("knx-message from {id}: measurement={value_string}, raw={:02X?}", filled_buf);
                        {
                            let mut f = self.log.lock().unwrap();
                            let now = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .expect("SystemTime::now()")
                                .as_secs();
                            match msg.measurement {
                                Some(m) => {
                                    let value = m.value.unwrap_or(0.);
                                    f.write_fmt(
                                        format_args!("{now};{id};{value}\n")).expect("write_fmt() to log");

                                    // store in-memory
                                    let mut data = self.data.lock().unwrap();
                                    match data.get_mut(id) {
                                        Some(mut store) => {
                                            store.timestamp = timestamp;
                                            store.value = m.value
                                        },
                                        None => eprintln!("sensor not known in data struct?")
                                    };
                                    data.insert(id, m.value.unwrap_or(0.0));
                                },
                                None => {
                                    // message without measurement
                                }
                            }
                        }
                    } else {
                        eprintln!("No handler for message to group-address {:?} (from {:?})", &msg.dst, &msg.src);
                    }
                }
                Err(()) => eprintln!("could not parse message.")
            }

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

impl Command {
    pub fn from_str(command_string: &str) -> Result<Command, String> {
        match command_string {
            "on" => Ok( Command::Switch(true) ),
            "off" => Ok( Command::Switch(false) ),
            "dim-0" => Ok( Command::Dimmer(0) ),
            "dim-5" => Ok( Command::Dimmer(5) ),
            "dim-10" => Ok( Command::Dimmer(10) ),
            "dim-25" => Ok( Command::Dimmer(25) ),
            "dim-50" => Ok( Command::Dimmer(50) ),
            "dim-100" => Ok( Command::Dimmer(100) ),
            "shutter-0" => Ok( Command::Shutter(1) ),
            "shutter-50" => Ok( Command::Shutter(50) ),
            "shutter-90" => Ok( Command::Shutter(90) ),
            "shutter-170" => Ok( Command::Shutter(170) ),
            "shutter-180" => Ok( Command::Shutter(180) ),
            "shutter-255" => Ok( Command::Shutter(255) ),
            s => {
                Err(format!("unknown string '{}'", s))
            }
        }
    }
}


pub fn create_knx_frame_dimmer(grp: u16, percent: u8) -> Result<SendMessage, ()>
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
    let percent_byte = percent as u16 * 255 / 100;
    v.append( &mut dst );
    v.push( 2u8 ); //len
    v.push( 0x00 ); // 'TPCI'
    v.push( 0x80 );
    v.push(percent_byte as u8);

    println!(" helligkeitswert: {:X?}", v);

    Ok( SendMessage { raw: v } )
}


//
// // 'full': entspricht wert, der fuer vollstaendiges schliessen gesendet werden muss, z.B. 205
fn create_knx_frame_rollo(grp: u16, percent: &u8) -> Result<SendMessage,()>
{
    let v = vec![
    // knx/ip header
    // HEADER_LEN: 0x06,  VERSION: 0x10 (1.0), ROUTING_INDIXATION (0x05, 0x03), TOTAL-LEN(, 18)
    0x06u8, 0x10, 0x05, 0x30, 0x00, 18u8,

         0x29u8, // data indication (rollo: 0x2e)
         0x00u8, // extra-info
         0xbcu8, //low-prio,
         0xe0u8, // to-group-address (1 << 7) | hop-count-6 (6 << 5) | extended-frame-format (0x0)
         0x12u8, 0x7eu8, // src-addr     ( 0x12, 0x7e -> 0x127e -> 1.2.126)
        (grp >> 8) as u8, (grp & 0xff) as u8,  // dst-addr: 0/0/22 -> 0x00 0x22 (Arbeit-Rollo)
         2u8, //len
         0u8,
         0x80u8,
         *percent
     ];

    Ok( SendMessage { raw: v } )
}


// slice: &[u8], array: &[u8; 2]
pub fn convert_16bit_float(high:u8, low:u8) -> f32 {
    // MEEEMMMM MMMMMMMM
    let mantissa = ((high & 0x07) as u16 * 256) + low as u16;
    let negative = high & 0x80 == 0x80;
    let exponent = match (high & 0x70) { 0 => 1, other => other as i32 };
    println!("negative={negative}, mantissa={mantissa}, exponent={exponent}");

    let mut out = (mantissa as f32).powi(exponent);
    if negative { out *= -1f32 };
    println!("--> {:02X}:{:02X} = {}", high, low, out);
    out * 0.02 // resolution: 0.01
}