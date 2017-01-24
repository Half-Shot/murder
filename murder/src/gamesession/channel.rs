#[derive(PartialEq)]
#[derive(Debug)]
pub enum EChannel {
    None = 0,
    Global = 1,
    Ghost = 2,
    Mafia = 3,
    Detective = 4,
}

pub enum EChannelMsgAction {
    NewPhase,
    VoteToLynch,
    VoteToMafiaKill,
    PlayerKilled,
    ClaimIsMafia,
    ClaimIsDetective,
    ClaimIsCiv,
}

pub struct ChannelMsg {
    from: usize,
    other: usize,
    channel: EChannel,
    action: EChannelMsgAction,
}

/*
How to read messages:
 * Messages are stored linearly in a vector
 * Each reader has a lastRead token.
 * AI/Clients will get a snapshot between then and now
 * AI will compute what is the best move to make based on that data.
*/
