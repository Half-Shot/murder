use gamesession::InternalState;
use slog::Logger;
use player::*;

pub struct SpecialState<'a> {
    pub state: &'a mut InternalState,
    log: Logger,
}

impl<'a> SpecialState<'a> {
    pub fn new(state: &'a mut InternalState) -> SpecialState<'a> {
        let log = state.logger.new(o!("context"=> "SpecialState"));
        SpecialState {
            state: state,
            log: log,
        }
    }

    pub fn detective_detect_role(&self, player: usize, subject: usize) -> Result<&PlayerRole,&str> {
        let plr : Option<&Player> = self.state.players.get(player);
        if plr.is_none() { // Player doesn't exist
            return Err("Player doesn't exist.");
        }
        else if plr.unwrap().is_ghost() {
            return Err("Player is ghost.");;
        }
        else if plr.unwrap().role() != &PlayerRole::Detective {
            return Err("Player is not a detective.");
        }
        let sub : Option<&Player> = self.state.players.get(subject);
        if sub.is_none() {
            return Err("Subject doesn't exist.");
        }
        else if plr.unwrap().is_ghost() {
            return Err("Subject is ghost.");;
        }
        return Ok(sub.unwrap().role());
    }
}

impl<'a> Drop for SpecialState<'a> {
    fn drop(&mut self) {
        // finalization of the actions for this state...
    }
}
