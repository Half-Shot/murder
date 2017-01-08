mod selection_state;
mod morning_state;
mod special_state;
mod mafia_state;
pub use self::selection_state::SelectionState as SelectionState;
pub use self::morning_state::MorningState as MorningState;
pub use self::special_state::SpecialState as SpecialState;
pub use self::mafia_state::MafiaState as MafiaState;

#[derive(PartialEq, Debug,Clone)]
pub enum EGamePhase {
    Selection,
    Morning,
    Special,
    Mafia,
}

pub enum GamePhase<'a> {
    Selection(SelectionState<'a>),
    Morning(MorningState<'a>),
    Special(SpecialState<'a>),
    Mafia(MafiaState<'a>),
}


impl<'a> PartialEq for GamePhase<'a> {
    fn eq(&self, other: &GamePhase<'a>) -> bool {
        return self == other;
    }
}
