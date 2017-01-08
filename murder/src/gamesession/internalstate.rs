use player::*;
use slog::Logger;
pub struct InternalState {
    pub players: Vec<Player>,
    pub mafia_kill: isize,
    pub first_night: bool,
    pub logger: Logger,
}
