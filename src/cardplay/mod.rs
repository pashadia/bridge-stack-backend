use crate::contract::BidContract;
use crate::Board;

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
    use crate::auction::StrainBid;
    use crate::cardplay::{Cardplay, PlayState};
    use crate::contract::{BidContract, Modifier};
    use crate::{Board, BridgeDirection};
    use std::convert::TryFrom;

    #[test]
    fn start_new_board() -> Result<(), ()> {
        let board = Board::new(3);
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
