use crate::contract::{BidContract, Contract, ContractLevel, Modifier, Strain};
use crate::{turns, BridgeDirection};
use num_traits::FromPrimitive;
use std::convert::TryFrom;

/// A bridge auction state machine
///
/// # Basic usage:
/// ```
/// # use bridge_backend::{Auction, BridgeDirection};
/// # use bridge_backend::auction::{Bid, StrainBid, PASS, ONE_NOTRUMP, THREE_NOTRUMP, Error};
/// # use std::convert::TryFrom;
/// # fn main() -> Result<(), Error> {
/// let mut auction = Auction::new(BridgeDirection::W);
///
/// auction.bid(ONE_NOTRUMP)?;
/// auction.bid(PASS)?;
/// auction.bid(THREE_NOTRUMP)?;
/// auction.bid(PASS)?;
/// auction.bid(PASS)?;
/// auction.bid(PASS)?;
///
/// assert!(auction.contract().is_some());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Auction {
    dealer: BridgeDirection,
    bids: Vec<Bid>,
    last_strain_bid: Option<StrainBid>,
    last_bidder: Option<BridgeDirection>,
}

impl Auction {
    /// Starts a new auction.
    ///
    /// The parameter received indicates the dealer. The internal `bids` vector is conceptually grouped in groups of four bids, starting with the dealer.
    pub fn new(dealer: BridgeDirection) -> Auction {
        Auction {
            dealer,
            bids: vec![],
            last_strain_bid: None,
            last_bidder: None,
        }
    }

    pub fn bid(&mut self, bid: Bid) -> Result<(), Error> {
        match bid {
            PASS => Ok(self.bids.push(bid)),
            Bid::RealBid(real_bid) => {
                if self.is_bid_sufficient(real_bid) {
                    self.last_strain_bid = Some(real_bid);
                    self.last_bidder = Some(self.whose_turn_is_it());
                    Ok(self.bids.push(bid))
                } else {
                    Err(Error::InsufficientBid)
                }
            }
            DOUBLE => {
                if self.can_double() {
                    Ok(self.bids.push(bid))
                } else {
                    Err(Error::CantDouble)
                }
            }
            REDOUBLE => {
                if self.can_redouble() {
                    Ok(self.bids.push(bid))
                } else {
                    Err(Error::CantRedouble)
                }
            }
        }
    }

    /// The auction is finished after everyone has bid at least once, and the last three bids were passes.
    pub fn is_completed(&self) -> bool {
        self.bids.len() >= 4 && self.bids.iter().rev().take(3).all(|&b| b == PASS)
    }

    /// Returns true if there's any other recorded bid but PASS.
    pub fn has_real_bid(&self) -> bool {
        self.bids.iter().any(|&b| b != PASS)
    }

    /// Ensures that the `StrainBid` received is a legal bid
    fn is_bid_sufficient(&self, other_bid: StrainBid) -> bool {
        match self.last_strain_bid {
            Some(this_bid) => other_bid > this_bid,
            None => true,
        }
    }

    /// Ensures that `DOUBLE` is a valid bid.
    fn can_double(&self) -> bool {
        if let Some(Bid::RealBid(_)) = self.last_meaningful_bid() {
            self.trailing_passes() != 1 // Can't double partner
        } else {
            false
        }
    }

    /// Ensures that `REDOUBLE` is a valid bid.
    fn can_redouble(&self) -> bool {
        if let Some(DOUBLE) = self.last_meaningful_bid() {
            self.trailing_passes() != 1 // Can't redouble partner
        } else {
            false
        }
    }

    /// Returns a clone of the last non-`PASS` bid.
    fn last_meaningful_bid(&self) -> Option<Bid> {
        self.bids.iter().rev().find(|&&b| b != PASS).cloned()
    }

    fn trailing_passes(&self) -> usize {
        self.bids.iter().rev().take_while(|&&b| b == PASS).count()
    }

    fn whose_turn_is_it(&self) -> BridgeDirection {
        let delta = self.bids.len() % 4;
        turns(self.dealer).skip(delta).next().unwrap()
    }

    /// Returns the `Contract` resulting from the `Auction`, when the auction is complete.
    pub fn contract(&self) -> Option<Contract> {
        if self.is_completed() {
            match self.last_strain_bid {
                None => Some(Contract::PassedOut),
                Some(contract) => {
                    let modifier: Modifier = match self
                        .last_meaningful_bid()
                        .expect("We should have a meaningful bid by now")
                    {
                        Bid::RealBid(_) => Modifier::Pass,
                        Bid::Other(modifier) => modifier,
                    };

                    let contract_set_by = self
                        .last_bidder
                        .expect("Bids have been made, we should have a bidder");
                    let declarer: BridgeDirection = self
                        .bids
                        .iter()
                        .zip(turns(self.dealer))
                        .filter_map(|(bid, bidder)| match bid {
                            Bid::RealBid(StrainBid { strain, .. })
                                if *strain == contract.strain =>
                            {
                                Some(bidder)
                            }
                            _ => None,
                        })
                        .find(|&bidder| {
                            bidder == contract_set_by || bidder == contract_set_by.partner()
                        })
                        .expect("Contracts tend to have a declarer");

                    Some(Contract::BidContract(BidContract {
                        contract,
                        modifier,
                        declarer,
                    }))
                }
            }
        } else {
            None
        }
    }
}

/// Represents a bid made by any player.
///
/// Bids are of two types:
///  - An overbid of a strain, which needs to be sufficient (a higher strain or level)
///  - a bid of PASS, DOUBLE or REDOUBLE, collectively known as `Other`(Bid::Other)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Bid {
    /// Represents a strain bid, either the first one, or an overbid of the previous one.
    RealBid(StrainBid),

    /// Represents a bid of PASS, DOUBLE or REDOUBLE.
    Other(Modifier),
}

/// Represents a Pass bid.
pub const PASS: Bid = Bid::Other(Modifier::Pass);

/// Represents a Double bid.
pub const DOUBLE: Bid = Bid::Other(Modifier::Double);

/// Represents a Redouble bid.
pub const REDOUBLE: Bid = Bid::Other(Modifier::Redouble);

/// Represents the bid of a strain by a player. Usually used through one of the named constants, e.g. [`ONE_CLUB`]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StrainBid {
    pub(crate) level: ContractLevel,
    pub(crate) strain: Strain,
}

impl TryFrom<&str> for StrainBid {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut chars = value.bytes();
        let level = chars
            .next()
            .map(|code| code - '0' as u8)
            .and_then(FromPrimitive::from_u8)
            .ok_or("Should be between 1 and 7")?;

        let strain = chars
            .next()
            .map(char::from)
            .as_ref()
            .map(char::to_ascii_uppercase)
            .and_then(|c| match c {
                'N' => Some(Strain::NoTrump),
                'S' => Some(Strain::Spades),
                'H' => Some(Strain::Hearts),
                'D' => Some(Strain::Diamonds),
                'C' => Some(Strain::Clubs),
                _ => None,
            })
            .ok_or("Should be either a suit or notrump")?;

        Ok(Self { level, strain })
    }
}

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

/// These are possible errors arising from trying to make a bid.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// An insufficient bid was attempted
    ///
    /// # Example:
    /// ```should_panic
    /// # use bridge_backend::{Auction, BridgeDirection};
    /// # use bridge_backend::auction::{Error, ONE_DIAMOND, ONE_CLUB};
    /// # fn main() -> Result<(), Error> {
    /// let mut auction = Auction::new(BridgeDirection::S);
    /// auction.bid(ONE_DIAMOND)?;
    ///
    /// // Attempting an insufficient bid panics
    /// auction.bid(ONE_CLUB)?;
    /// # Ok(())
    /// # }
    /// ```
    InsufficientBid,

    /// An illegal Double was attempted
    ///
    /// # Example:
    /// ```should_panic
    /// # use bridge_backend::{Auction, BridgeDirection};
    /// use bridge_backend::auction::{Bid::RealBid, Error, StrainBid, PASS, DOUBLE, ONE_CLUB};
    /// use std::convert::TryFrom;
    /// # fn main() -> Result<(), Error> {
    /// let mut auction = Auction::new(BridgeDirection::S);
    /// auction.bid(ONE_CLUB)?;
    /// auction.bid(PASS)?;
    ///
    /// // Attempting to double partner is illegal
    /// auction.bid(DOUBLE)?;
    /// # Ok(())
    /// # }
    /// ```
    CantDouble,

    /// An illegal Redouble was attempted
    ///
    /// # Example:
    /// ```should_panic
    /// # use bridge_backend::{Auction, BridgeDirection};
    /// use bridge_backend::auction::{Bid::RealBid, Error, StrainBid, PASS, REDOUBLE, ONE_DIAMOND};
    /// use std::convert::TryFrom;
    /// # fn main() -> Result<(), Error> {
    /// let mut auction = Auction::new(BridgeDirection::S);
    /// auction.bid(ONE_DIAMOND)?;
    ///
    /// // Attempting to redouble without a double is illegal
    /// auction.bid(REDOUBLE)?;
    /// # Ok(())
    /// # }
    /// ```
    CantRedouble,
}

#[cfg(test)]
mod tests;
