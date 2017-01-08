use ai::AiTrait;
use phase::*;

pub struct AiDumb{
    pub player_id: usize
}

impl AiDumb {

}

impl AiTrait for AiDumb {
    fn phase_morning(&self, state: &mut MorningState) {
        println!("I'm just dumb", );

    }
    fn phase_special(&self, state: &mut SpecialState) {

    }
    fn phase_mafia(&self, state: &mut MafiaState) {

    }
}
