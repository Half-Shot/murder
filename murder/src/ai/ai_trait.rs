use phase::*;

pub trait AiTrait {
    fn phase_morning(&self, state: &mut MorningState);
    fn phase_special(&self, state: &mut SpecialState);
    fn phase_mafia(&self, state: &mut MafiaState);
}
