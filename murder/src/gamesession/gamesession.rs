use gamesession::channel::*;
use player::*;
use InternalState;
use phase::*;
use slog::*;

pub struct GameSession {
    phase: EGamePhase,
    state: InternalState,
    log: Logger,
}

impl GameSession {
    pub fn new() -> GameSession {
        let root = Logger::root(Discard, o!(
            "murder.version" => env!("CARGO_PKG_VERSION"),
        ));
        let log = root.new(o!("context"=> "GameSession"));

        GameSession {
            phase: EGamePhase::Selection,
            state: InternalState {
                players: Vec::new(),
                mafia_kill: -1,
                first_night: true,
                logger: root,
            },
            log: log
         }
    }

    pub fn use_logger(&mut self, logger: Logger){
        self.state.logger = logger;
        self.log = self.state.logger.new(o!("context"=> "GameSession"));
    }

    pub fn get_player(& self, i : usize) -> &Player {
        return self.state.players.get(i).expect("Couldn't find player.");
    }

    pub fn get_players(&self) -> &Vec<Player> {
        return &self.state.players;
    }

    pub fn current_phase(&mut self) -> GamePhase {
        match self.phase {
            EGamePhase::Selection => return GamePhase::Selection(SelectionState::new(&mut self.state)),
            EGamePhase::Morning => return GamePhase::Morning(MorningState::new(&mut self.state)),
            EGamePhase::Special => return GamePhase::Special(SpecialState::new(&mut self.state)),
            EGamePhase::Mafia => return GamePhase::Mafia(MafiaState::new(&mut self.state)),
        }
    }

    pub fn advance_phase(&mut self) {
        // This is yuck, but that's rust.
        let old_phase = self.phase.clone();
        match self.phase {
            EGamePhase::Selection => self.phase = EGamePhase::Morning,
            EGamePhase::Morning => self.phase = EGamePhase::Special,
            EGamePhase::Special => self.phase = EGamePhase::Mafia,
            EGamePhase::Mafia => self.phase = EGamePhase::Morning,
        }
        info!(self.log, "Moved from {:?} to {:?}", old_phase, self.phase);
    }

    pub fn player_channels(&self, i: usize) -> Vec<EChannel> {
        let player : &Player = &self.get_player(i);
        let mut channels : Vec<EChannel> = Vec::new();
        if player.is_ghost() {
            channels.push(EChannel::Ghost);
            return channels;
        }

        match self.phase {
            EGamePhase::Morning | EGamePhase::Selection => {
                channels.push(EChannel::Global);
            }
            EGamePhase::Special => {
                if player.role() == &PlayerRole::Detective {
                    channels.push(EChannel::Detective);
                }
            }
            EGamePhase::Mafia => {
                if player.role() == &PlayerRole::Mafia {
                    channels.push(EChannel::Mafia);
                }
            }
        }

        if channels.len() == 0 {
            channels.push(EChannel::None);
        }

        return channels;
    }
}
