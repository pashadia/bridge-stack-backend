#![allow(dead_code)]

mod contract;
use contract::Contract;

mod auction;
use auction::Auction;

mod cardplay;

use bridge_deck::Cards;
use cardplay::Cardplay;

pub struct Board {
    pub north: Cards,
    pub east: Cards,
    pub south: Cards,
    pub west: Cards,
    number: usize,
}

impl Board {
    pub fn new() -> Self {
        Self::new_with_number(1)
    }

    pub fn new_with_number(number: usize) -> Self {
        let mut full_deck = Cards::ALL;

        Self {
            north: full_deck.pick(13).expect("Should be able to get 13 cards"),
            east: full_deck.pick(13).expect("Should be able to get 13 cards"),
            south: full_deck.pick(13).expect("Should be able to get 13 cards"),
            west: full_deck.pick(13).expect("Should be able to get 13 cards"),
            number,
        }
    }

    pub fn vulnerability(self) -> Vulnerability {
        match self.number % 16 {
            1 | 8 | 11 | 14 => Vulnerability::NONE,
            2 | 5 | 12 | 15 => Vulnerability::NS,
            3 | 6 | 9 | 0 => Vulnerability::EW,
            _ => Vulnerability::ALL,
        }
    }

    pub fn dealer(self) -> BridgeDirection {
        match self.number % 4 {
            1 => BridgeDirection::N,
            2 => BridgeDirection::E,
            3 => BridgeDirection::S,
            _ => BridgeDirection::W,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum BridgeDirection {
    N,
    E,
    S,
    W,
}

impl BridgeDirection {
    fn partner(&self) -> BridgeDirection {
        match self {
            BridgeDirection::N => BridgeDirection::S,
            BridgeDirection::E => BridgeDirection::W,
            BridgeDirection::S => BridgeDirection::N,
            BridgeDirection::W => BridgeDirection::E,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Vulnerability {
    NS,
    EW,
    ALL,
    NONE,
}

#[derive(Debug)]
pub struct Turns {
    last: BridgeDirection,
}
impl Iterator for Turns {
    type Item = BridgeDirection;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.last;
        self.last = match self.last {
            BridgeDirection::N => BridgeDirection::E,
            BridgeDirection::E => BridgeDirection::S,
            BridgeDirection::S => BridgeDirection::W,
            BridgeDirection::W => BridgeDirection::N,
        };
        Some(res)
    }
}
pub fn turns(dealer: BridgeDirection) -> Turns {
    Turns { last: dealer }
}

impl Vulnerability {
    pub fn is_vulnerable(self, who: BridgeDirection) -> bool {
        match self {
            Vulnerability::NS => [BridgeDirection::N, BridgeDirection::S].contains(&who),
            Vulnerability::EW => [BridgeDirection::E, BridgeDirection::W].contains(&who),
            Vulnerability::ALL => true,
            Vulnerability::NONE => false,
        }
    }
}

pub(crate) struct BoardPlay {
    board: Board,
    state: BoardState,
    table_number: usize,
    contract: Option<Contract>,
    tricks_taken: usize,
}

impl BoardPlay {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            state: Default::default(),
            table_number: 0,
            contract: None,
            tricks_taken: 0,
        }
    }

    pub fn start_play(self) {}

    pub fn score(self) -> Option<i32> {
        match self.state {
            BoardState::Completed => Some(
                self.contract?
                    .get_score_for_tricks(self.tricks_taken, self.board.vulnerability()),
            ),
            _ => None,
        }
    }
}

enum BoardState {
    NotStarted,
    Bidding(Auction),
    OnLead(Auction),
    Playing(Auction, Contract, Cardplay),
    Completed,
}

impl Default for BoardState {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[cfg(test)]
mod tests {
    use crate::bridge::{turns, BridgeDirection};

    mod board_creation {
        use crate::bridge::{Board, BridgeDirection, Vulnerability};

        #[test]
        fn new_board() {
            let board = Board::new();
            assert_eq!(board.number, 1);

            let board = Board::new_with_number(7);
            assert_eq!(board.number, 7);
        }

        #[test]
        fn all_cards_should_exist() {
            let board = Board::new();
            let cards = board
                .north
                .union(board.east)
                .union(board.south)
                .union(board.west);
            assert_eq!(cards.len(), 52)
        }

        #[test]
        fn correct_number_of_cards() {
            let board = Board::new();
            assert_eq!(board.north.len(), 13);
            assert_eq!(board.east.len(), 13);
            assert_eq!(board.south.len(), 13);
            assert_eq!(board.west.len(), 13);
        }

        #[test]
        fn vulnerability() {
            assert_eq!(
                Board::new_with_number(7).vulnerability(),
                Vulnerability::ALL
            );
            assert_eq!(
                Board::new_with_number(99).vulnerability(),
                Vulnerability::EW
            );
        }

        #[test]
        fn dealer() {
            assert_eq!(Board::new().dealer(), BridgeDirection::N);
            assert_eq!(Board::new_with_number(2).dealer(), BridgeDirection::E);
            assert_eq!(Board::new_with_number(31).dealer(), BridgeDirection::S);
            assert_eq!(Board::new_with_number(136).dealer(), BridgeDirection::W);
        }
    }

    #[test]
    fn test_turns() {
        let mut t = turns(BridgeDirection::N);
        assert_eq!(t.next(), Some(BridgeDirection::N));
        assert_eq!(t.next(), Some(BridgeDirection::E));
        assert_eq!(t.next(), Some(BridgeDirection::S));
        assert_eq!(t.next(), Some(BridgeDirection::W));
        assert_eq!(t.next(), Some(BridgeDirection::N));
        assert_eq!(t.next(), Some(BridgeDirection::E));

        let mut tw = turns(BridgeDirection::W);
        assert_eq!(tw.next(), Some(BridgeDirection::W));
        assert_eq!(tw.next(), Some(BridgeDirection::N));
        assert_eq!(tw.next(), Some(BridgeDirection::E));
        assert_eq!(t.next(), Some(BridgeDirection::S));
        assert_eq!(t.next(), Some(BridgeDirection::W));
        assert_eq!(t.next(), Some(BridgeDirection::N));
    }
}
