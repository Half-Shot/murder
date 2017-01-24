extern crate murder;
extern crate rand;
#[macro_use]
extern crate slog;
extern crate slog_term;
use std::io;
use murder::*;
use slog::*;

fn setup_logger() -> Logger {
    let drain = slog_term::streamer().async().full().build();
    Logger::root(LevelFilter::new(drain, Level::Debug).fuse(), o!(
        "version" => env!("CARGO_PKG_VERSION"),
    ))

}

fn main() {
    let root = setup_logger();
    let log = root.new(o!{"context"=>"client"});
    run_session(&root, &log);
}

fn run_session(root: &Logger, log: &Logger) {
    let mut session = GameSession::new();
    let mut ai_players : Vec<Box<AiTrait>> = Vec::new();
    let mut input = String::new();
    let plr_names = vec![
    "The Ducktor",
    "A.B.U.S.E",
    "/sbin/init",
    "Red Max",
    "MaxwullKopler",
    "WierdAI",
    "systemd4lyfe",
    "Bacon Man",
    "Triloboat",
    "Ghost in the bash",
    "MuricanHero",
    "NotTheMafia",
    "ImWithDetective",
    "LynchMeBro",
    "FatWaffle"
    ];
    session.use_logger(root.new(o!{}));
    loop {
        match session.current_phase() {
            murder::GamePhase::Selection(mut state) => {
                for i in 0..15 {
                    state.add_player(plr_names[i].to_string());
                }
                info!(log, "Assigning roles");
                state.assign_roles();
                for i in 0..15 {
                    match state.get_player_role(i) {
                        &PlayerRole::Civilian => {
                            debug!(log, "Added Civilian AI");
                            ai_players.push(Box::new(AiCivilian {
                                player_id: i
                            }));
                        },
                        &PlayerRole::Mafia => {
                            debug!(log, "Added Mafia AI");
                            ai_players.push(Box::new(AiMafia {
                                player_id: i
                            }));
                        }
                        _ => {
                            debug!(log, "Added Dumb AI");
                            ai_players.push(Box::new(AiDumb {
                                player_id: i
                            }));
                        }
                    }
                }

            },
            murder::GamePhase::Morning(mut state) => {
                match state.has_team_won() {
                    murder::PlayerRole::Civilian => {
                        info!(log,"Civilians win!");
                        break;
                    }
                    murder::PlayerRole::Mafia => {
                        info!(log,"Mafia win!");
                        break;
                    }
                    _ => {
                        //Nobody has won yet
                    }
                }

                for ai_player in &ai_players {
                    ai_player.phase_morning(&mut state);
                }

                if state.can_anyone_vote() {
                    let target : Option<&Player> = state.lynch_target();
                    if target.is_some() {
                        let player : &Player = target.unwrap();
                        info!(log,"{:?} was lynched. They were a {:?}", player.name(), player.role() );
                    }
                    else {
                        info!(log,"Nobody was killed this round.");
                    }
                }
                else {
                    info!(log,"No kills before first night!");
                }
            },
            murder::GamePhase::Special(mut state) => {
                for ai_player in &ai_players {
                    ai_player.phase_special(&mut state);
                }
            },
            murder::GamePhase::Mafia(mut state) => {
                for ai_player in &ai_players {
                    ai_player.phase_mafia(&mut state);
                }
            }
        }
        io::stdin().read_line(&mut input);
        session.advance_phase();
    }
}
