use std::net::TcpStream;
use slog::*;
use std::io::Write;
use slog_scope;
use packet;
use packet::RequestType;
use pb_murder::*;
use uuid::Uuid;
use protobuf::Message;
use murder;
use packetphase;


pub struct GameServer {
    log : Logger,
    session: murder::GameSession,
    uuid : String,
    has_started : bool,
}

impl GameServer {
    pub fn new() -> GameServer {
        let uuid = Uuid::new_v4().simple().to_string();
        GameServer {
            log: slog_scope::logger().new(o!{"context"=>"GameServer"}),
            uuid : uuid,
            has_started: false,
            session: murder::GameSession::new()
        }
    }

    pub fn run<'a>(&mut self, mut stream : TcpStream) {
        self.session.use_logger(self.log.new(o!{"context"=>"GameSession"}));
        slog_info!(self.log, "Got connection.");
        loop {
            let res = packet::get_packet(&mut stream);
            if res.is_err() {
                slog_warn!(self.log,"Dropping client due to read error: {:?}", &res.err().unwrap() );
                break;
            }
            let mut packet = res.unwrap();

            if packet.request_type == RequestType::UNKNOWN {
                let mut err_pkt = packet::encode_packet(packet::RequestType::UNKNOWN, vec![1]);
                stream.write_all(err_pkt.as_mut_slice()).unwrap();
                continue;
            }

            if packet.request_type == RequestType::HEARTBEAT {
                let mut pkt = packet::encode_packet(RequestType::HEARTBEAT, vec![1]);
                stream.write_all(pkt.as_mut_slice()).unwrap();
                continue;
            }

            if !self.has_started {
                if packet.request_type == RequestType::NEWGAME {
                    slog_info!(self.log,"Got new game request!");
                    let mut proto = SrvNewGame::new();
                    proto.set_uuid(self.uuid.clone());
                    proto.set_has_started(self.has_started);
                    self.has_started = true;
                    let mut pkt = packet::encode_packet(RequestType::NEWGAME,proto.write_to_bytes().unwrap());
                    stream.write_all(pkt.as_mut_slice()).unwrap();
                }
                else {
                    let mut err_pkt = packet::encode_packet(
                        packet::RequestType::ERROR,packetphase::get_error(
                        "N_A_TO_STATE".to_string(),
                        "This request is not applicable to this state.".to_string()
                    ));
                    stream.write_all(err_pkt.as_mut_slice()).unwrap();
                }
                continue;
            }


            let response : packetphase::PacketResponse = match self.session.current_phase() {
                murder::GamePhase::Selection(mut state) => {
                    packetphase::phase_selection(&mut stream,&mut state, &mut packet)
                },
                murder::GamePhase::Morning(mut state) => {
                    packetphase::phase_morning(&mut stream,&mut state, &mut packet)
                },
                murder::GamePhase::Special(mut state) => {
                    packetphase::phase_special(&mut stream, &mut state, &mut packet)
                },
                murder::GamePhase::Mafia(mut state) => {
                    packetphase::phase_mafia(&mut stream, &mut state, &mut packet)
                }
            };

            if response.should_advance() {
                self.session.advance_phase();
            }

            if response.not_applicable_state()  {
                let mut err_pkt = packet::encode_packet(
                    packet::RequestType::ERROR,packetphase::get_error(
                    "N_A_TO_STATE".to_string(),
                    "This request is not applicable to this state.".to_string()
                ));
                stream.write_all(err_pkt.as_mut_slice()).unwrap();
            }
        }
        //TODO: Hold state somewhere when we drop connection.
        slog_info!(self.log, "Dropped connection.");
    }
}
