use crate::bridge::contract::{BidContract, Contract, ContractLevel, Modifier, Strain};
use crate::bridge::BridgeDirection;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Auction {
    dealer: BridgeDirection,
    bids: Vec<Bid>,
    last_strain_bid: Bid,
}

impl Auction {
    pub fn new(dealer: BridgeDirection) -> Auction {
        Auction {
            dealer,
            bids: vec![],
            last_strain_bid: PASS,
        }
    }

    pub fn bid(&mut self, bid: Bid) -> Result<(), Error> {
        match bid {
            PASS => Ok(self.bids.push(bid)),
            Bid::RealBid(_) => {
                if self.is_bid_sufficient(bid) {
                    self.last_strain_bid = bid;
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

    pub fn is_completed(&self) -> bool {
        if self.has_real_bid() {
            self.bids.iter().rev().take(3).all(|&b| b == PASS)
        } else {
            self.bids.len() > 3
        }
    }

    pub fn has_real_bid(&self) -> bool {
        self.bids.iter().any(|&b| b != PASS)
    }

    fn is_bid_sufficient(&self, bid: Bid) -> bool {
        self.last_strain_bid == PASS || bid == PASS || bid > self.last_strain_bid
    }

    fn can_double(&self) -> bool {
        if let Some(Bid::RealBid(_)) = self.last_meaningful_bid() {
            self.trailing_passes() != 1 // Can't double partner
        } else {
            false
        }
    }

    fn can_redouble(&self) -> bool {
        if let Some(DOUBLE) = self.last_meaningful_bid() {
            self.trailing_passes() != 1 // Can't redouble partner
        } else {
            false
        }
    }

    fn last_meaningful_bid(&self) -> Option<Bid> {
        self.bids.iter().rev().find(|&&b| b != PASS).cloned()
    }

    fn trailing_passes(&self) -> usize {
        self.bids.iter().rev().take_while(|&&b| b == PASS).count()
    }

    pub fn contract(&self) -> Option<Contract> {
        if self.is_completed() {
            match self.last_meaningful_bid() {
                None => Some(Contract::PassedOut),
                Some(bid) => {
                    let modifier = match bid {
                        Bid::RealBid(_) => Modifier::Pass,
                        Bid::Other(modifier) => modifier,
                    };
                    let declarer = self.dealer;
                    if let Bid::RealBid(strain_bid) = self.last_strain_bid {
                        Some(Contract::BidContract(BidContract {
                            strain: strain_bid.strain,
                            level: strain_bid.level,
                            modifier,
                            declarer,
                        }))
                    } else {
                        unreachable!("last strain should really be a bid")
                    }
                }
            }
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Bid {
    RealBid(StrainBid),
    Other(Modifier),
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match other {
            Bid::RealBid(other_real) => match self {
                Bid::RealBid(this_real) => this_real.partial_cmp(other_real),
                Bid::Other(_) => Some(Ordering::Greater),
            },
            Bid::Other(_) => Some(Ordering::Less),
        }
    }
}

const PASS: Bid = Bid::Other(Modifier::Pass);
const DOUBLE: Bid = Bid::Other(Modifier::Double);
const REDOUBLE: Bid = Bid::Other(Modifier::Redouble);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StrainBid {
    level: ContractLevel,
    strain: Strain,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InsufficientBid,
    CantDouble,
    CantRedouble,
}

#[cfg(test)]
mod tests;
