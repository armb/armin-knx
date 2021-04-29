
#[derive(Debug, Copy, Clone, PartialEq)]
struct Address(u8, u8, u8);


pub struct Knx {
  socket: std::net::UdpSocket,
}


pub struct Message {
  data: Vec<u8>
}

pub struct Error {
  message: String
}

pub fn create() -> Knx {
  let socket = std::net::UdpSocket::bind("0.0.0.0:0").expect("bind local udp socket somewhere");

  let mut k: Knx = Knx { socket: socket };

  k.connect();

  k
}


impl Knx {
  // 'constructor'
  
  pub fn connect(&mut self) -> Result<(),String> {
    match self.socket.connect("224.0.23.12:3671") {
      Ok(_) => Ok(()),
      Err(_) => Err("could not set udp multicast address for send()".into())
     }
  }

  pub fn generate_on_off(grp: u16, onoff: bool) -> Message {

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

    let m = Message { data: v };
    m
  }




  pub fn create_knx_frame_dimmer(grp: u16, percent: u8) -> Vec<u8>
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

    v
    }
}
  