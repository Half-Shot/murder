use ai::AiTrait;
use phase::*;

pub struct AiCivilian{
    pub player_id: usize
}

impl AiCivilian {

}

impl AiTrait for AiCivilian {
    fn phase_morning(&self, state: &mut MorningState) {

    }
    fn phase_special(&self, state: &mut SpecialState) {

    }
    fn phase_mafia(&self, state: &mut MafiaState) {

    }
}
