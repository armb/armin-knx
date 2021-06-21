use std::sync::{Arc, Mutex};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
struct Address(u8, u8, u8);

pub enum Command {
    Dimmer(u8), //percent
    Switch(bool),
    UpDownTarget(u8,u8), //value,max
}


pub struct Knx {
    socket: std::net::UdpSocket,
}


pub struct Message {
    raw: Vec<u8>
}


pub fn create() -> Knx {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").expect("bind() failed");

    socket.join_multicast_v4(
        &std::net::Ipv4Addr::from_str("224.0.23.12").unwrap(),
//        &std::net::Ipv4Addr::from_str("192.168.0.90").unwrap()).expect("join_multicast_v4()");
 	&std::net::Ipv4Addr::from_str("192.168.0.208").unwrap()).expect("join_multicast_v4()");


    socket.connect("224.0.23.12:3671").expect("connect() failed");
    
    let k: Knx = Knx { socket: socket };

    k
}



impl Knx {

    // pub fn sendRaw(&mut self, msg: Message) {
    // 	self.socket.send(msg.raw.as_slice()).expect("send() failed");
    // }

    pub fn send(&mut self, grp: u16, command: Command) {
	let msg = match command {
	    Command::Dimmer(x) => create_knx_frame_dimmer(grp, x),
	    Command::Switch(x) => create_knx_frame_onoff(grp, x),
	    Command::UpDownTarget(x,max) => create_knx_frame_rollo(grp, x, max)
	};
	
	self.socket.send(msg.raw.as_slice()).expect("send() failed");
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

    let m = Message { raw: v };
    m
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

    Message { raw: v }
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

    Message { raw:v }
}

