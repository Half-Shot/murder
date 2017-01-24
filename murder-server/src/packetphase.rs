use std::net::TcpStream;
use murder;
use pb_murder::*;
use protobuf::{Message, RepeatedField};
use packet;
use packet::{RequestType, MurderPacket};
use std::io::Write;

const MIN_PLAYERS : usize = 8;

pub struct PacketResponse {
    should_advance : bool,
    not_applicable_state : bool,
}

impl PacketResponse {
    pub fn should_advance(&self) -> bool {
        self.should_advance
    }
    pub fn not_applicable_state(&self) -> bool {
        self.not_applicable_state
    }
}

pub fn phase_selection(stream: &mut TcpStream, state: &mut murder::SelectionState, packet: &mut MurderPacket) -> PacketResponse {
    match packet.request_type {
        RequestType::ADDPLAYER => {
            let mut pbc_new_player = CliAddPlayer::new();
            packet.get_message(&mut pbc_new_player);
            let p_id = state.add_player(pbc_new_player.get_player_name().to_string());
            let mut pbs_new_player = SrvAddPlayer::new();
            pbs_new_player.set_player_id(p_id as u32);
            let mut pkt = packet::encode_packet(RequestType::ADDPLAYER,pbs_new_player.write_to_bytes().unwrap());
            stream.write_all(pkt.as_mut_slice()).unwrap();
            PacketResponse {
                should_advance: false,
                not_applicable_state: false,
            }
        },
        RequestType::STATE => {
            let mut pbs_state = SrvState::new();
            pbs_state.set_state(0);
            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
            stream.write_all(pkt.as_mut_slice()).unwrap();
            PacketResponse {
                should_advance: false,
                not_applicable_state: false,
            }
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
                return PacketResponse {
                    should_advance: true,
                    not_applicable_state: false,
                }
            }
            PacketResponse {
                should_advance: false,
                not_applicable_state: false,
            }
        }
        _ =>  PacketResponse { should_advance: false, not_applicable_state: true,},
    }
}

pub fn phase_morning(stream: &mut TcpStream, state: &mut murder::MorningState, packet: &mut MurderPacket) -> PacketResponse {
    match packet.request_type {
        RequestType::STATE => {
            let mut pbs_state = SrvState::new();
            pbs_state.set_state(1);
            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
            stream.write_all(pkt.as_mut_slice()).unwrap();
            PacketResponse { should_advance: false, not_applicable_state: false, }
        },
        RequestType::ADVANCE => {
            if state.can_anyone_vote() {
                let mut pbs_vote = SrvVote::new();
                info!("Forcing a lynching!");
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
            PacketResponse {
                should_advance: true,
                not_applicable_state: false,
            }
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
            PacketResponse {
                should_advance: pbs_vote.finished,
                not_applicable_state: false,
            }
        }
        _ =>  PacketResponse { should_advance: false, not_applicable_state: true,},
    }
}

pub fn phase_special(stream: &mut TcpStream, state: &mut murder::SpecialState, packet: &mut MurderPacket) -> PacketResponse {
    match packet.request_type {
        RequestType::STATE => {
            let mut pbs_state = SrvState::new();
            pbs_state.set_state(2);
            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
            stream.write_all(pkt.as_mut_slice()).unwrap();
            PacketResponse { should_advance: false, not_applicable_state: false, }
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
            PacketResponse { should_advance: false, not_applicable_state: false, }
        }
        _ =>  PacketResponse { should_advance: false, not_applicable_state: true,},
    }
}

pub fn phase_mafia(stream: &mut TcpStream, state: &mut murder::MafiaState, packet: &mut MurderPacket) -> PacketResponse {
    match packet.request_type {
        RequestType::VOTE => {
            let mut pbc_vote = CliVote::new();
            packet.get_message(&mut pbc_vote);
            PacketResponse { should_advance: false, not_applicable_state: false, }
        }
        RequestType::STATE => {
            let mut pbs_state = SrvState::new();
            pbs_state.set_state(3);
            let mut pkt = packet::encode_packet(RequestType::STATE,pbs_state.write_to_bytes().unwrap());
            stream.write_all(pkt.as_mut_slice()).unwrap();
            PacketResponse { should_advance: false, not_applicable_state: false, }
        },
        _ =>  PacketResponse { should_advance: false, not_applicable_state: true,},
    }
}

pub fn get_error(error: String, details : String) -> Vec<u8> {
    let mut er = SrvError::new();
    er.set_error(error);
    er.set_details(details);
    er.write_to_bytes().unwrap()
}
