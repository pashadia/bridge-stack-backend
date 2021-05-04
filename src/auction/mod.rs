//! This module defines the rules of a Bridge auction
//!
//! Its' main struct is [`Auction`] which defines the bridge auction state machine. See its documentation for an usage example.

use std::convert::TryFrom;

use num_traits::FromPrimitive;

use constants::*;

use crate::contract::{BidContract, Contract, ContractLevel, Modifier, Strain};
use crate::{turns, BridgeDirection};

/// A bridge auction state machine
///
/// # Basic usage:
/// ```
/// # use bridge_backend::{Auction, BridgeDirection};
/// # use bridge_backend::auction::{Error, constants::*};
///
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

    /// Represents a bid made by the current player.
    ///
    /// Returns `Ok(())` if the bid is sufficient and accepted. It returns an `auction::Error` variant otherwise.
    /// # Example:
    /// ```
    /// # use bridge_backend::{Auction, BridgeDirection};
    /// # use bridge_backend::auction::constants::*;
    /// let mut auction = Auction::new(BridgeDirection::S);
    /// auction.bid(TWO_SPADES)?;
    /// auction.bid(DOUBLE)?;
    /// auction.bid(REDOUBLE)?;
    /// auction.bid(PASS)?;
    /// ```
    ///
    /// Note: By definition, the bid is made by the player whose turn it is. Out of turn bids are impossible to model.
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

pub mod constants;

/// These are possible errors arising from trying to make a bid.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// An insufficient bid was attempted
    ///
    /// # Example:
    /// ```should_panic
    /// # use bridge_backend::{Auction, BridgeDirection};
    /// # use bridge_backend::auction::{Error, constants::*};
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
    /// # use bridge_backend::auction::{Error, constants::*};
    ///
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
    /// # use bridge_backend::auction::{Error, constants::*};
    ///
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
