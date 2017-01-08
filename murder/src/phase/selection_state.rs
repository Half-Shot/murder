use gamesession::InternalState;
use rand::{thread_rng, Rng};
use player::*;
use slog::Logger;

pub struct SelectionState<'a> {
    state: &'a mut InternalState,
    log: Logger,
}

impl<'a> SelectionState<'a> {

    pub fn new(state: &'a mut InternalState) -> SelectionState<'a> {
        let log = state.logger.new(o!("context"=> "SelectionState"));
        SelectionState {
            state: state,
            log: log,
        }
    }

    pub fn add_player(&mut self, name: String) -> usize {
        self.state.players.push(
            Player::new(name)
        );
        return self.state.players.len()-1;
    }

    pub fn get_player_role(&self, plr: usize) -> &PlayerRole {
        return self.state.players.get(plr).unwrap().role();
    }

    pub fn drop_player(&mut self, plr: usize) {
        self.state.players.swap_remove(plr);
    }

    // Call this last.
    pub fn assign_roles(&mut self) {
        /*
        Number of roles.
        4 civilians per mafia.
        8 players per special.

        A game of 13 will have 9 civlians, 3 mafia and one special.
        */
        let mut roles: Vec<PlayerRole> = Vec::with_capacity(self.state.players.len());

        for i in 1..(self.state.players.len()+1) {
            if i % 9 == 0 {//Special
                roles.push(PlayerRole::Detective);
            }
            else if i % 4 == 0 {
                roles.push(PlayerRole::Mafia);
            }
            else {
                roles.push(PlayerRole::Civilian);
            }
        }

        let mut rng = thread_rng();
        rng.shuffle(&mut roles[..]);
        for player in &mut self.state.players {
            let role = roles.pop().expect("Couldn't find a role for user.");
            player.assign_role(role);
            debug!(self.log, "Player {:?} has role {:?}", player.name(), player.role() );
        }
    }


}

impl<'a> Drop for SelectionState<'a> {
    fn drop(&mut self) {
        // finalization of the actions for this state...
    }
}
