use crate::bridge::contract::BidContract;
use crate::bridge::Board;

mod trick;
use trick::CompletedTrick;

pub struct Cardplay {
    tricks: Vec<CompletedTrick>,
    state: PlayState,
}

impl Cardplay {
    fn start(_board: &Board, _contract: BidContract) -> Self {
        Self {
            tricks: vec![],
            state: PlayState::BeforeLead,
        }
    }

    fn tricks_played(&self) -> usize {
        self.tricks.len()
    }
}

#[derive(Eq, PartialEq, Debug)]
enum PlayState {
    BeforeLead,
}

#[cfg(test)]
mod tests {
    use crate::bridge::auction::StrainBid;
    use crate::bridge::cardplay::{Cardplay, PlayState};
    use crate::bridge::contract::{BidContract, Modifier};
    use crate::bridge::{Board, BridgeDirection};
    use std::convert::TryFrom;

    #[test]
    fn start_new_board() -> Result<(), ()> {
        let board = Board::new_with_number(3);
        let contract = BidContract {
            contract: StrainBid::try_from("2h").unwrap(),
            modifier: Modifier::Pass,
            declarer: BridgeDirection::N,
        };
        let play = Cardplay::start(&board, contract);
        assert_eq!(play.tricks_played(), 0);
        assert_eq!(play.state, PlayState::BeforeLead);

        Ok(())
    }
}
