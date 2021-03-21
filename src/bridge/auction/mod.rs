use crate::bridge::contract::{BidContract, Contract, ContractLevel, Modifier, Strain};
use crate::bridge::BridgeDirection;
use num_traits::FromPrimitive;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Auction {
    dealer: BridgeDirection,
    bids: Vec<Bid>,
    last_strain_bid: Option<StrainBid>,
}

impl Auction {
    pub fn new(dealer: BridgeDirection) -> Auction {
        Auction {
            dealer,
            bids: vec![],
            last_strain_bid: None,
        }
    }

    pub fn bid(&mut self, bid: Bid) -> Result<(), Error> {
        match bid {
            PASS => Ok(self.bids.push(bid)),
            Bid::RealBid(real_bid) => {
                if self.is_bid_sufficient(real_bid) {
                    self.last_strain_bid = Some(real_bid);
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

    fn is_bid_sufficient(&self, other_bid: StrainBid) -> bool {
        match self.last_strain_bid {
            Some(this_bid) => other_bid > this_bid,
            None => true,
        }
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
            match self.last_strain_bid {
                None => Some(Contract::PassedOut),
                Some(contract) => {
                    let modifier: Modifier = match self.last_meaningful_bid() {
                        None => unreachable!("We should have a meaningful bid by now"),
                        Some(last_bid) => match last_bid {
                            Bid::RealBid(_) => Modifier::Pass,
                            Bid::Other(modifier) => modifier,
                        },
                    };
                    let declarer = self.dealer; // FIXME
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Bid {
    RealBid(StrainBid),
    Other(Modifier),
}

const PASS: Bid = Bid::Other(Modifier::Pass);
const DOUBLE: Bid = Bid::Other(Modifier::Double);
const REDOUBLE: Bid = Bid::Other(Modifier::Redouble);

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

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InsufficientBid,
    CantDouble,
    CantRedouble,
}

#[cfg(test)]
mod tests;
