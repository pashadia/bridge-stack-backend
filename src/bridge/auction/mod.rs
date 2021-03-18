use crate::bridge::contract::{Contract, ContractLevel, Strain};
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
        if self.is_bid_sufficient(bid) {
            if let Bid::RealBid(_) = bid {
                self.last_strain_bid = bid;
            };
            Ok(self.bids.push(bid))
        } else {
            Err(Error::InsufficientBid)
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

    pub fn contract(&self) -> Option<Contract> {
        if self.is_completed() {
            Some(Contract::PassedOut)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Bid {
    Pass,
    RealBid(StrainBid),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StrainBid {
    level: ContractLevel,
    strain: Strain,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InsufficientBid,
}

#[cfg(test)]
mod tests;
