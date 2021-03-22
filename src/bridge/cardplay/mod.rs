use crate::bridge::contract::BidContract;
use crate::bridge::Board;

pub struct Cardplay;

impl Cardplay {
    fn start(_board: &Board, _contract: BidContract) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use crate::bridge::auction::StrainBid;
    use crate::bridge::cardplay::Cardplay;
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
        dbg!(&board.inner.east.len());
        let _play = Cardplay::start(&board, contract);
        Ok(())
    }
}
