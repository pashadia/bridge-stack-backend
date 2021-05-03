#![allow(dead_code)]
#![warn(missing_docs)]
#![deny(missing_crate_level_docs)]

//! A state machine for the Bridge card game.

mod contract;
use contract::Contract;

mod auction;
use auction::Auction;

mod cardplay;

use bridge_deck::Cards;
use cardplay::Cardplay;

/// Represents a bridge board.
///
/// It holds all the static state of a board: the cards held by all players at the beginning, and the board's number. Not to be mistaken with [`BoardPlay`] which tracks the state of a board when played at a specific table.
pub struct Board {
    /// The cards held by North
    pub north: Cards,
    /// The cards held by East
    pub east: Cards,
    /// The cards held by South
    pub south: Cards,
    /// The cards held by West
    pub west: Cards,
    number: usize,
}

impl Board {
    /// Generates the first board. Mostly used for testing.
    pub fn first() -> Self {
        Self::new(1)
    }

    /// Generates a single board.
    ///
    /// It takes a single parameter for the board number. The dealer and the vulnerability are based on it.
    pub fn new(number: usize) -> Self {
        let mut full_deck = Cards::ALL;

        Self {
            north: full_deck.pick(13).expect("Should be able to get 13 cards"),
            east: full_deck.pick(13).expect("Should be able to get 13 cards"),
            south: full_deck.pick(13).expect("Should be able to get 13 cards"),
            west: full_deck.pick(13).expect("Should be able to get 13 cards"),
            number,
        }
    }

    /// Returns this board's vulnerability, according to the rules of the game
    ///
    /// ```
    /// use bridge_backend::{Board, Vulnerability};
    ///
    /// assert_eq!(Board::new(7).vulnerability(), Vulnerability::ALL);
    /// assert_eq!(Board::new(99).vulnerability(), Vulnerability::EW);
    /// ```
    pub fn vulnerability(self) -> Vulnerability {
        match self.number % 16 {
            1 | 8 | 11 | 14 => Vulnerability::NONE,
            2 | 5 | 12 | 15 => Vulnerability::NS,
            3 | 6 | 9 | 0 => Vulnerability::EW,
            _ => Vulnerability::ALL,
        }
    }

    /// Returns this board's dealer, according to the rules of the game
    ///
    /// ```
    /// use bridge_backend::{Board, BridgeDirection};
    ///
    /// assert_eq!(Board::first().dealer(), BridgeDirection::N);
    /// assert_eq!(Board::new(2).dealer(), BridgeDirection::E);
    /// assert_eq!(Board::new(31).dealer(), BridgeDirection::S);
    /// assert_eq!(Board::new(136).dealer(), BridgeDirection::W);
    /// ```
    pub fn dealer(self) -> BridgeDirection {
        match self.number % 4 {
            1 => BridgeDirection::N,
            2 => BridgeDirection::E,
            3 => BridgeDirection::S,
            _ => BridgeDirection::W,
        }
    }
}

/// Represents a specific position at a bridge table.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum BridgeDirection {
    /// North
    N,
    /// East
    E,
    /// South
    S,
    /// West
    W,
}

impl BridgeDirection {
    /// Returns the partner of a specific player.
    fn partner(&self) -> BridgeDirection {
        match self {
            BridgeDirection::N => BridgeDirection::S,
            BridgeDirection::E => BridgeDirection::W,
            BridgeDirection::S => BridgeDirection::N,
            BridgeDirection::W => BridgeDirection::E,
        }
    }
}

/// An iterator that returns the natural turns of a bridge game.
///
/// This `struct` is created by the [`turns()`] function. See its documentation for more.
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

/// Creates a new iterator that shows the natural turns of a bridge game.
///
/// It takes a single parameter for the first player to take action, and then continues endlessly.
///
/// # Example
/// ```
/// use bridge_backend::{turns, BridgeDirection};
///
/// let mut tw = turns(BridgeDirection::W);
/// assert_eq!(tw.next(), Some(BridgeDirection::W));
/// assert_eq!(tw.next(), Some(BridgeDirection::N));
/// assert_eq!(tw.next(), Some(BridgeDirection::E));
/// assert_eq!(tw.next(), Some(BridgeDirection::S));
/// assert_eq!(tw.next(), Some(BridgeDirection::W));
/// assert_eq!(tw.next(), Some(BridgeDirection::N));
/// ```
pub fn turns(dealer: BridgeDirection) -> Turns {
    Turns { last: dealer }
}

/// A struct which represents a bridge board vulnerability.
///
/// It is created by the [`vulnerability`](method@Board::vulnerability) method on a [Board].
#[derive(Eq, PartialEq, Debug)]
pub enum Vulnerability {
    /// North-South vulnerable
    NS,

    /// East-West vulnerable
    EW,

    /// Both sides vulnerable
    ALL,

    /// No side vulnerable
    NONE,
}
impl Vulnerability {
    /// Utility function to test the vulnerability of a specific player.
    pub fn is_vulnerable(self, who: BridgeDirection) -> bool {
        match self {
            Vulnerability::NS => [BridgeDirection::N, BridgeDirection::S].contains(&who),
            Vulnerability::EW => [BridgeDirection::E, BridgeDirection::W].contains(&who),
            Vulnerability::ALL => true,
            Vulnerability::NONE => false,
        }
    }
}

/// Represents the state of a bridge board.
pub struct BoardPlay {
    board: Board,
    state: BoardState,
    table_number: usize,
    contract: Option<Contract>,
    tricks_taken: usize,
}

impl BoardPlay {
    /// Creates a new `BoardPlay` with default values. Todo: Replace with the `Default` trait.
    pub fn new() -> Self {
        Self {
            board: Board::first(),
            state: Default::default(),
            table_number: 0,
            contract: None,
            tricks_taken: 0,
        }
    }

    /// Calculates the score for the board.
    ///
    /// The score is returned from the perspective of North-South, in accordance to the real-world standard set by other software.
    ///
    /// Returns `None` when the board is not completed yet.
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
    use crate::Board;

    #[test]
    fn new_board() {
        let board = Board::first();
        assert_eq!(board.number, 1);

        let board = Board::new(7);
        assert_eq!(board.number, 7);
    }

    #[test]
    fn all_cards_should_exist() {
        let board = Board::first();
        let cards = board
            .north
            .union(board.east)
            .union(board.south)
            .union(board.west);
        assert_eq!(cards.len(), 52)
    }

    #[test]
    fn correct_number_of_cards() {
        let board = Board::first();
        assert_eq!(board.north.len(), 13);
        assert_eq!(board.east.len(), 13);
        assert_eq!(board.south.len(), 13);
        assert_eq!(board.west.len(), 13);
    }
}
