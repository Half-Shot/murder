use gamesession::InternalState;
use player::*;
use slog::Logger;

pub struct MafiaState<'a> {
    state: &'a mut InternalState,
    log: Logger,
}

impl<'a> MafiaState<'a> {

    pub fn new(state: &'a mut InternalState) -> MafiaState<'a> {
        let log = state.logger.new(o!("context"=> "MafiaState"));
        MafiaState {
            state: state,
            log: log,
        }
    }

    fn can_kill(&self, killer_id: usize) -> bool {
        let k : Option<&Player> = self.state.players.get(killer_id);
        if k.is_none() {
            return false;
        }
        let killer = k.unwrap();
        return killer.role() == &PlayerRole::Mafia && !killer.is_ghost();
    }

    pub fn mafia_vote(&mut self, killer_id: usize, victim_id: usize) -> Result<&Player,&str> {
        if !self.can_kill(killer_id) {
            return Err("Killer wasn't allowed to kill.")
        }

        let v : Option<&mut Player> = self.state.players.get_mut(killer_id);
        if v.is_none() {
            return Err("Victim didn't exist.");
        }
        let victim = v.unwrap();

        if !victim.is_ghost() {
            //self.kill_vote
            return Ok(victim);
            // victim.kill();
            // self.state.mafia_kill = victim_id as isize;
            // return Ok(victim)
        }
        return Err("Couldn't kill the victim.");
    }
}

impl<'a> Drop for MafiaState<'a> {
    fn drop(&mut self) {
        // finalization of the actions for this state...
    }
}
