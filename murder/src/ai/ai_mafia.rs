use ai::AiTrait;
use phase::*;

pub struct AiMafia{
    pub player_id: usize
}

impl AiMafia {

}

impl AiTrait for AiMafia {
    fn phase_morning(&self, state: &mut MorningState) {
        println!("I'm an evil mafioso", );

    }
    fn phase_special(&self, state: &mut SpecialState) {

    }
    fn phase_mafia(&self, state: &mut MafiaState) {

    }
}
