use std::net::TcpStream;
use slog::*;
use std::io::Write;
use slog_scope;
use packet;
use packet::RequestType;
use pb_murder::*;
use uuid::Uuid;
use protobuf::{Message, RepeatedField};
use murder;

const MIN_PLAYERS : usize = 8;

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
            let mut na_state = false;
            let mut uk_type = false;
            let mut should_advance = false;

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
                        packet::RequestType::ERROR,get_error(
                        "N_A_TO_STATE".to_string(),
                        "This request is not applicable to this state.".to_string()
                    ));
                    stream.write_all(err_pkt.as_mut_slice()).unwrap();
                }
                continue;
            }

            match self.session.current_phase() {
                murder::GamePhase::Selection(mut state) => {
                    match packet.request_type {
                        RequestType::ADDPLAYER => {
                            let mut pbc_new_player = CliAddPlayer::new();
                            packet.get_message(&mut pbc_new_player);
                            let p_id = state.add_player(pbc_new_player.get_player_name().to_string());
                            let mut pbs_new_player = SrvAddPlayer::new();
                            pbs_new_player.set_player_id(p_id as u32);
                            let mut pkt = packet::encode_packet(RequestType::ADDPLAYER,pbs_new_player.write_to_bytes().unwrap());
                            stream.write_all(pkt.as_mut_slice()).unwrap();
                        },
                        RequestType::STATE => {
                            let mut pbs_state = SrvState::new();
                            pbs_state.set_state(0);
                            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
                            stream.write_all(pkt.as_mut_slice()).unwrap();
                        },
                        RequestType::ADVANCE => {
                            if state.get_player_count() < MIN_PLAYERS {
                                let mut err_pkt = packet::encode_packet(
                                    packet::RequestType::ERROR,get_error(
                                    "NOT_ENOUGH_PLAYERS".to_string(),
                                    "You need a minimum of 8 players.".to_string()
                                ));
                                stream.write_all(err_pkt.as_mut_slice()).unwrap();
                            }
                            else {
                                state.assign_roles();
                                let mut pbs_roles = SrvRoles::new();
                                let mut roles = RepeatedField::new();
                                for i in 0..state.get_player_count() {
                                    roles.push(state.get_player_role(i).to_string());
                                }
                                pbs_roles.set_role(roles);
                                let mut pkt = packet::encode_packet(RequestType::ROLES,pbs_roles.write_to_bytes().unwrap());
                                stream.write_all(pkt.as_mut_slice()).unwrap();
                                should_advance = true;
                            }
                        }
                        RequestType::UNKNOWN => uk_type = true,
                        _ => na_state = true,
                    }
                },
                murder::GamePhase::Morning(mut state) => {
                    match packet.request_type {
                        RequestType::STATE => {
                            let mut pbs_state = SrvState::new();
                            pbs_state.set_state(1);
                            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
                            stream.write_all(pkt.as_mut_slice()).unwrap();
                        },
                        RequestType::ADVANCE => {
                            if state.can_anyone_vote() {
                                let mut pbs_vote = SrvVote::new();
                                slog_info!(self.log, "Forcing a lynching!");
                                let target = state.lynch_target();
                                if target.is_some() {
                                    pbs_vote.lynched = target.unwrap() as u32;
                                } else {
                                    pbs_vote.lynched = 255 as u32;
                                }
                                let mut pkt = packet::encode_packet(
                                    RequestType::VOTE,
                                    pbs_vote.write_to_bytes().unwrap()
                                );
                                stream.write_all(pkt.as_mut_slice()).unwrap();
                            }
                            should_advance = true;
                        },
                        RequestType::VOTE => {
                            let mut pbc_vote = CliVote::new();
                            let mut pbs_vote = SrvVote::new();
                            packet.get_message(&mut pbc_vote);

                            {
                                let pick = state.pick_target(pbc_vote.sender as usize, pbc_vote.victim as usize);
                                if pick.is_err() {
                                    let mut err_pkt = packet::encode_packet(
                                        packet::RequestType::ERROR,get_error(
                                        "COULD_NOT_VOTE".to_string(),
                                        pick.err().unwrap().to_string()
                                    ));
                                    stream.write_all(err_pkt.as_mut_slice()).unwrap();
                                }
                                pbs_vote.votes = pick.unwrap() as u32;
                            }

                            pbs_vote.finished = state.counted_all_votes();
                            if state.counted_all_votes() {
                                let target = state.lynch_target();
                                if target.is_some() {
                                    pbs_vote.lynched = target.unwrap() as u32;
                                } else {
                                    pbs_vote.lynched = 255 as u32;
                                }
                            }

                            let mut pkt = packet::encode_packet(
                                RequestType::VOTE,
                                pbs_vote.write_to_bytes().unwrap()
                            );
                            stream.write_all(pkt.as_mut_slice()).unwrap();
                        }
                        RequestType::UNKNOWN => uk_type = true,
                        _ => na_state = true,
                    }
                },
                murder::GamePhase::Special(mut state) => {
                    match packet.request_type {
                        RequestType::STATE => {
                            let mut pbs_state = SrvState::new();
                            pbs_state.set_state(2);
                            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
                            stream.write_all(pkt.as_mut_slice()).unwrap();
                        },
                        RequestType::DETECTIVEINVESTIGATE => {
                            let mut pbc_di = CliDetectiveInvestigate::new();
                            packet.get_message(&mut pbc_di);
                            let res = state.detective_detect_role(pbc_di.sender as usize, pbc_di.target as usize);
                            if res.is_ok() {
                                let mut pbs_di = SrvDetectiveInvestigate::new();
                                pbs_di.target = pbc_di.target;
                                pbs_di.role = res.unwrap().to_string();
                                let mut pkt = packet::encode_packet(
                                    RequestType::DETECTIVEINVESTIGATE,
                                    pbs_di.write_to_bytes().unwrap()
                                );
                                stream.write_all(pkt.as_mut_slice()).unwrap();
                            }
                            else {
                                let mut err_pkt = packet::encode_packet(
                                    packet::RequestType::ERROR,get_error(
                                    "COULD_NOT_PERFORM_ACTION".to_string(),
                                    res.err().unwrap().to_string()
                                ));
                                stream.write_all(err_pkt.as_mut_slice()).unwrap();
                            }
                        }
                        RequestType::UNKNOWN => uk_type = true,
                        _ => na_state = true,
                    }
                },
                murder::GamePhase::Mafia(mut state) => {
                    match packet.request_type {
                        RequestType::STATE => {
                            let mut pbs_state = SrvState::new();
                            pbs_state.set_state(3);
                            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
                            stream.write_all(pkt.as_mut_slice()).unwrap();
                        },
                        RequestType::UNKNOWN => uk_type = true,
                        _ => na_state = true,
                    }
                }
            }

            if should_advance {
                self.session.advance_phase();
            }

            if uk_type {
                let mut err_pkt = packet::encode_packet(packet::RequestType::UNKNOWN, vec![1]);
                stream.write_all(err_pkt.as_mut_slice()).unwrap();
            }

            if na_state  {
                let mut err_pkt = packet::encode_packet(
                    packet::RequestType::ERROR,get_error(
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

fn get_error(error: String, details : String) -> Vec<u8> {
    let mut er = SrvError::new();
    er.set_error(error);
    er.set_details(details);
    er.write_to_bytes().unwrap()
}
