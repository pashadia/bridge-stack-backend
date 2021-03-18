use crate::bridge::contract::Contract;
use crate::bridge::contract::Contract::PassedOut;
use crate::bridge::BridgeDirection;

#[derive(Debug)]
pub struct Auction {
    dealer: BridgeDirection,
    bids: Vec<Bid>,
}

impl Auction {
    pub fn new(dealer: BridgeDirection) -> Auction {
        Auction {
            dealer,
            bids: vec![],
        }
    }

    pub fn bid(&mut self, bid: Bid) {
        self.bids.push(bid);
    }

    pub fn is_completed(&self) -> bool {
        if self.has_real_bid() {
            todo!()
        } else {
            self.bids.len() > 3
        }
    }

    pub fn has_real_bid(&self) -> bool {
        false
    }

    pub fn contract(&self) -> Option<Contract> {
        if self.is_completed() {
            Some(PassedOut)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum Bid {
    Pass,
}

#[cfg(test)]
mod tests;
