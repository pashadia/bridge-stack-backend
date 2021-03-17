use crate::bridge::contract::{BidContract, ContractModifier, Strain};

pub fn trick_score(strain: Strain, how_many: usize) -> usize {
    match strain {
        Strain::Clubs | Strain::Diamonds => 20 * how_many,
        Strain::Hearts | Strain::Spades => 30 * how_many,
        Strain::NoTrump => 30 * how_many + 10,
    }
}

pub fn over_score(contract: &BidContract, over: usize, vul: bool) -> usize {
    match contract.modifier {
        ContractModifier::Passed => match contract.suit {
            Strain::Clubs | Strain::Diamonds => 20 * over,
            _ => 30 * over,
        },
        ContractModifier::Doubled => {
            if vul {
                200 * over
            } else {
                100 * over
            }
        }
        ContractModifier::Redoubled => {
            if vul {
                400 * over
            } else {
                200 * over
            }
        }
    }
}
