//! Defines all possible bids in the auction in the game of Bridge.

use crate::auction::{Bid, StrainBid};
use crate::contract::{ContractLevel, Modifier, Strain};

macro_rules! make_bid {
    ($name:ident, $level:ident, $strain:ident) => {
        #[doc = "Constant representing the named bid."]
        pub const $name: Bid = Bid::RealBid(StrainBid {
            level: ContractLevel::$level,
            strain: Strain::$strain,
        });
    };
}

make_bid!(ONE_CLUB, One, Clubs);
make_bid!(ONE_DIAMOND, One, Diamonds);
make_bid!(ONE_HEART, One, Hearts);
make_bid!(ONE_SPADE, One, Spades);
make_bid!(ONE_NOTRUMP, One, NoTrump);

make_bid!(TWO_CLUBS, Two, Clubs);
make_bid!(TWO_DIAMONDS, Two, Diamonds);
make_bid!(TWO_HEARTS, Two, Hearts);
make_bid!(TWO_SPADES, Two, Spades);
make_bid!(TWO_NOTRUMP, Two, NoTrump);

make_bid!(THREE_CLUBS, Three, Clubs);
make_bid!(THREE_DIAMONDS, Three, Diamonds);
make_bid!(THREE_HEARTS, Three, Hearts);
make_bid!(THREE_SPADES, Three, Spades);
make_bid!(THREE_NOTRUMP, Three, NoTrump);

make_bid!(FOUR_CLUBS, Four, Clubs);
make_bid!(FOUR_DIAMONDS, Four, Diamonds);
make_bid!(FOUR_HEARTS, Four, Hearts);
make_bid!(FOUR_SPADES, Four, Spades);
make_bid!(FOUR_NOTRUMP, Four, NoTrump);

make_bid!(FIVE_CLUBS, Five, Clubs);
make_bid!(FIVE_DIAMONDS, Five, Diamonds);
make_bid!(FIVE_HEARTS, Five, Hearts);
make_bid!(FIVE_SPADES, Five, Spades);
make_bid!(FIVE_NOTRUMP, Five, NoTrump);

make_bid!(SIX_CLUBS, Six, Clubs);
make_bid!(SIX_DIAMONDS, Six, Diamonds);
make_bid!(SIX_HEARTS, Six, Hearts);
make_bid!(SIX_SPADES, Six, Spades);
make_bid!(SIX_NOTRUMP, Six, NoTrump);

make_bid!(SEVEN_CLUBS, Seven, Clubs);
make_bid!(SEVEN_DIAMONDS, Seven, Diamonds);
make_bid!(SEVEN_HEARTS, Seven, Hearts);
make_bid!(SEVEN_SPADES, Seven, Spades);
make_bid!(SEVEN_NOTRUMP, Seven, NoTrump);

/// Represents a Pass bid.
pub const PASS: Bid = Bid::Other(Modifier::Pass);

/// Represents a Double bid.
pub const DOUBLE: Bid = Bid::Other(Modifier::Double);

/// Represents a Redouble bid.
pub const REDOUBLE: Bid = Bid::Other(Modifier::Redouble);
