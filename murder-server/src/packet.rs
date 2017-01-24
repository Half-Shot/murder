use std::net::TcpStream;
use std::io::{Error, Read};
use protobuf::{Message, CodedInputStream};

const HEADER_SIZE : usize = 10;
const MAGIC_BYTE_B1 : u8 = 0x25;
const MAGIC_BYTE_B2 : u8 = 0xC9;
const MAGIC_BYTE_B3 : u8 = 0xC3;
const MAGIC_BYTE_B4 : u8 = 0x5F;
const SERVER_VERSION : u8 = 0x00;

#[derive(Debug, PartialEq)]
pub enum RequestType {
    HEARTBEAT = 0,
    NEWGAME = 1,
    ADDPLAYER = 2,
    ADVANCE = 4,
    STATE = 5,
    ROLES = 6,
    VOTE = 7,
    CHATCHANNELS = 8,
    DETECTIVEINVESTIGATE = 9,
    FAILURE = 200,
    ERROR = 201,
    UNKNOWN = 255,
}

impl RequestType {
    pub fn from_byte(byte: u8) -> RequestType {
        match byte {
            0 => RequestType::HEARTBEAT,
            1 => RequestType::NEWGAME,
            2 => RequestType::ADDPLAYER,
            4 => RequestType::ADVANCE,
            5 => RequestType::STATE,
            6 => RequestType::ROLES,
            7 => RequestType::VOTE,
            8 => RequestType::CHATCHANNELS,
            9 => RequestType::DETECTIVEINVESTIGATE,
            200 => RequestType::FAILURE,
            _ => RequestType::UNKNOWN,
        }
    }
}

#[derive(Debug)]
pub struct MurderPacket {
    pub request_type : RequestType,
    pub payload : Vec<u8>,
}

impl MurderPacket {
    pub fn get_message<T : Message >(&mut self, msg : &mut T) {
        let mut is = CodedInputStream::from_bytes(self.payload.as_mut_slice());
        msg.merge_from(&mut is).unwrap();//TODO: Return as error.
    }
}

pub fn encode_packet(request_type: RequestType, payload: Vec<u8>) -> Vec<u8>{
    let mut buf = Vec::new();
    buf.resize(HEADER_SIZE, 0);
    buf[0] = MAGIC_BYTE_B1;
    buf[1] = MAGIC_BYTE_B2;
    buf[2] = MAGIC_BYTE_B3;
    buf[3] = MAGIC_BYTE_B4;
    buf[4] = SERVER_VERSION;
    buf[5] = request_type as u8;
    for i in 0..4 {
        buf[6+i] = (payload.len() >> (24 - i*8)) as u8;
    }
    buf.extend(&payload);
    return buf;
}

pub fn get_packet(stream: &mut TcpStream) -> Result<MurderPacket,String> {
    let packet_res = read_until_len(stream, HEADER_SIZE);
    if packet_res.is_err(){
        return Err(packet_res.err().unwrap().to_string());
    }
    let packet = packet_res.unwrap();

    let header_res = check_header(&packet); // Will also fail here if the buffer doesn't get filled.
    if header_res.is_err() {
         return Err(header_res.err().unwrap().to_string());
    }

    let request_type = packet[5];
    let mut pl_size : u64 = 0;
    for i in 0..4 {
        pl_size += (packet[9-i] as u64) << 8*i;
    }
    let payload = read_until_len(stream, pl_size as usize);
    if payload.is_err() {
         return Err(payload.err().unwrap().to_string());
    }
    return Ok(MurderPacket {
        request_type: RequestType::from_byte(request_type),
        payload: payload.unwrap(),
    });
}


fn check_header (head: &Vec<u8>) -> Result<(),&str> {
    if head[0] != MAGIC_BYTE_B1 ||
       head[1] != MAGIC_BYTE_B2 ||
       head[2] != MAGIC_BYTE_B3 ||
       head[3] != MAGIC_BYTE_B4 {
           return Err("Invalid magic bytes");
    }
    if head[4] != SERVER_VERSION {
        return Err("Client supported version does not match server version.");;
    }
    return Ok(());
}

fn read_until_len(stream: &mut TcpStream, len : usize) -> Result<Vec<u8>,Error>{
    let mut buf = Vec::new();
    buf.resize(len, 0);
    let mut read = 0;
    while read < len {
        let res = stream.read(buf.as_mut_slice());
        if res.is_err() {
            return Err(res.err().unwrap());
        }
        if read == 0 {
            break;
        }
        read += res.unwrap();
    }
    return Ok(buf);
}
