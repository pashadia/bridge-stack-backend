use crate::bridge::contract::{BidContract, Contract, ContractLevel, ContractModifier, Strain};
use crate::bridge::BridgeDirection;

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
            last_strain_bid: Bid::Pass,
        }
    }

    pub fn bid(&mut self, bid: Bid) -> Result<(), Error> {
        match bid {
            Bid::Pass => Ok(self.bids.push(bid)),
            Bid::RealBid(_) => {
                if self.is_bid_sufficient(bid) {
                    self.last_strain_bid = bid;
                    Ok(self.bids.push(bid))
                } else {
                    Err(Error::InsufficientBid)
                }
            }
            Bid::Double => {
                if self.can_double() {
                    Ok(self.bids.push(bid))
                } else {
                    Err(Error::CantDouble)
                }
            }
            Bid::Redouble => {
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
            self.bids.iter().rev().take(3).all(|&b| b == Bid::Pass)
        } else {
            self.bids.len() > 3
        }
    }

    pub fn has_real_bid(&self) -> bool {
        self.bids.iter().any(|&b| b != Bid::Pass)
    }

    fn is_bid_sufficient(&self, bid: Bid) -> bool {
        self.last_strain_bid == Bid::Pass || bid == Bid::Pass || bid > self.last_strain_bid
    }

    fn can_double(&self) -> bool {
        if let Some(Bid::RealBid(_)) = self.last_meaningful_bid() {
            self.trailing_passes() != 1 // Can't double partner
        } else {
            false
        }
    }

    fn can_redouble(&self) -> bool {
        if let Some(Bid::Double) = self.last_meaningful_bid() {
            self.trailing_passes() != 1 // Can't redouble partner
        } else {
            false
        }
    }

    fn last_meaningful_bid(&self) -> Option<Bid> {
        self.bids.iter().rev().find(|&&b| b != Bid::Pass).cloned()
    }

    fn trailing_passes(&self) -> usize {
        self.bids
            .iter()
            .rev()
            .take_while(|&&b| b == Bid::Pass)
            .count()
    }

    pub fn contract(&self) -> Option<Contract> {
        if self.is_completed() {
            match self.last_meaningful_bid() {
                None => Some(Contract::PassedOut),
                Some(bid) => {
                    let modifier = match bid {
                        Bid::Pass => {
                            unreachable!()
                        }
                        Bid::RealBid(_) => ContractModifier::Passed,
                        Bid::Double => ContractModifier::Doubled,
                        Bid::Redouble => ContractModifier::Redoubled,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Bid {
    Pass,
    RealBid(StrainBid),
    Double,
    Redouble,
}

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
