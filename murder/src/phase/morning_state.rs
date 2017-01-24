use gamesession::InternalState;
use std::collections::HashMap;
use player::*;
use slog::Logger;
const LINCH_PERCENTAGE : f32 = 0.4;

#[derive(PartialEq,Debug)]
enum LynchState {
    Picked,
    Abstained,
}
#[derive(Debug)]
struct LynchPlayerState {
    picked: usize,
    state: LynchState,
}

pub struct MorningState<'a> {
    state: &'a mut InternalState,
    lynch_state: HashMap<usize, LynchPlayerState>,
    log: Logger
}

impl<'a> MorningState<'a> {

    pub fn new(state: &'a mut InternalState) -> MorningState<'a> {
        let log = state.logger.new(o!("context"=> "MorningState"));
        MorningState {
            state: state,
            lynch_state : HashMap::new(),
            log: log,
        }
    }

    pub fn can_vote(&mut self, player: usize) -> bool {
        if self.state.first_night { // Ignore first night
            warn!(self.log, "{:?} tried to vote in the first night!", player);
            return false;
        }
        let plr : Option<&Player> = self.state.players.get(player);
        if plr.is_none() { // Player doesn't exist
            error!(self.log, "{:?} tried to vote, but doesn't exist?!?", player);
            return false;
        }
        if plr.unwrap().is_ghost() {
            return false;
        }
        !self.lynch_state.contains_key(&player)
    }

    pub fn has_team_won(&self) -> PlayerRole {
        let mut mafia = 0;
        let mut civ = 0;
        let mut lone_wolf = 0;
        for plr in &self.state.players {
            if plr.is_ghost(){
                continue;
            }
            if plr.role() == &PlayerRole::Mafia {
                mafia += 1;
            }
            else {
                civ += 1;
            }
        }
        debug!(self.log, "CURRENT_SCORE";"CIV"=>civ, "MAFIA"=>mafia);
        if mafia == 0 {
            return PlayerRole::Civilian;
        }
        if mafia > civ {
            return PlayerRole::Mafia;
        }
        else {
            return PlayerRole::Unassigned;
        }
    }

    pub fn abstain(&mut self, player: usize) {
        if !self.can_vote(player) {
            warn!(self.log, "{:?} tried to abstain, but can't vote.", player);
            return;
        }
        self.lynch_state.insert(player, LynchPlayerState {
            picked: 0,
            state: LynchState::Abstained,
        });
    }

    pub fn pick_target(&mut self, player: usize, victim: usize) -> Result<usize,&str> {
        if !self.can_vote(player) {
            return Err("Player cannot vote.");
        }
        let plr : Option<&Player> = self.state.players.get(victim);
        if plr.is_none() {
            error!(self.log, "{:?} tried to vote for {:?}, but they do not exist", player, victim);
            return Err("Victim not found.");
        }
        if plr.unwrap().is_ghost() {
            error!(self.log, "{:?} tried to vote for {:?}, but they are a ghostie", player, victim);
            return Err("Victim is a ghost.");
        }
        self.lynch_state.insert(player, LynchPlayerState {
            picked: victim,
            state: LynchState::Picked,
        });
        Ok(self.lynch_state.len())

    }

    pub fn counted_all_votes(&self) -> bool {
        return self.state.players.len() == self.lynch_state.len();
    }

    pub fn can_anyone_vote(&self) -> bool {
        !self.state.first_night
    }

    pub fn lynch_target(&mut self) -> Option<usize> {
        let mut votes : HashMap<usize,usize> = HashMap::with_capacity(self.lynch_state.len());
        for vote in self.lynch_state.values() {
            if vote.state == LynchState::Picked {
                let count = votes.entry(vote.picked).or_insert(0);
                *count += 1;
            }
        }
        let mut sorted_votes : Vec<_> = votes.iter().collect();
        sorted_votes.sort_by(|a, b| b.1.cmp(a.1));
        debug!(self.log, "Votes: {:?}", sorted_votes);
        for (player, vote) in sorted_votes {
            let percentage = (*vote as f32) / (self.lynch_state.len() as f32);
            debug!(self.log, "{:?} got {:?} votes ({:?})", player, vote, percentage);
            if percentage >= LINCH_PERCENTAGE {
                //TODO: Add a defense stage
                let plr = self.state.players.get_mut(*player).expect("Couldn't find player to lynch");
                debug!(self.log, "Killed {:?} ({:?})", player, plr.name());
                plr.kill();
                return Some(*player);
            }
        }
        None
    }
}

impl<'a> Drop for MorningState<'a> {
    fn drop(&mut self) {
        self.state.first_night = false; // Before we leave, make sure the first night mod is off.
    }
}
