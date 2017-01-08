use gamesession::InternalState;
use slog::Logger;
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
}

impl<'a> Drop for SpecialState<'a> {
    fn drop(&mut self) {
        // finalization of the actions for this state...
    }
}
