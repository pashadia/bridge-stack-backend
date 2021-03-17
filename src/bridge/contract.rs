use crate::bridge::{BridgeDirection, Vulnerability};
use std::cmp::max;

pub enum Contract {
    PassedOut,
    BidContract(BidContract),
}

impl Contract {
    pub fn get_score_for_tricks(&self, tricks_taken: usize, vulnerability: Vulnerability) -> i32 {
        match self {
            Contract::PassedOut => 0,
            Contract::BidContract(actual_contract) => {
                let tricks_needed: usize = 6 + actual_contract.level as usize;

                let vul = vulnerability.is_vulnerable(actual_contract.declarer);
                if tricks_needed > tricks_taken {
                    let down = tricks_taken as i32 - tricks_needed as i32;

                    match actual_contract.modifier {
                        ContractModifier::Passed => {
                            let base_value = if vul { 100 } else { 50 };
                            base_value * down
                        }
                        ContractModifier::Doubled => {
                            if vul {
                                down * 300 + 100
                            } else {
                                let bad = if down < -1 { max(-2, down + 1) } else { 0 };
                                let worse = if down < -3 { down + 3 } else { 0 };
                                worse * 300 + bad * 200 - 100
                            }
                        }
                        ContractModifier::Redoubled => {
                            if vul {
                                down * 600 + 200
                            } else {
                                let bad = if down < -1 { max(-2, down + 1) } else { 0 };
                                let worse = if down < -3 { down + 3 } else { 0 };
                                worse * 600 + bad * 400 - 200
                            }
                        }
                    }
                } else {
                    let over = tricks_needed - tricks_taken;
                    dbg!(over);
                    0
                }
            }
        }
    }
}

pub struct BidContract {
    suit: Strain,
    pub(crate) level: ContractLevel,
    modifier: ContractModifier,
    declarer: BridgeDirection,
}

pub enum Strain {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    NoTrump,
}

#[derive(Copy, Clone)]
pub enum ContractLevel {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

pub enum ContractModifier {
    Passed,
    Doubled,
    Redoubled,
}

#[cfg(test)]
mod tests {

    use crate::bridge::contract::{BidContract, Contract, ContractLevel, ContractModifier, Strain};
    use crate::bridge::{BridgeDirection, Vulnerability};

    #[test]
    fn new() {
        let _contract = BidContract {
            suit: Strain::Spades,
            level: ContractLevel::Four,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::S,
        };
    }

    #[test]
    fn score_undoubled_down() {
        let bid = BidContract {
            suit: Strain::Spades,
            level: ContractLevel::Four,
            modifier: ContractModifier::Passed,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(bid);

        assert_eq!(contract.get_score_for_tricks(4, Vulnerability::ALL), -600);
        assert_eq!(contract.get_score_for_tricks(5, Vulnerability::ALL), -500);
        assert_eq!(contract.get_score_for_tricks(6, Vulnerability::ALL), -400);
        assert_eq!(contract.get_score_for_tricks(7, Vulnerability::ALL), -300);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::ALL), -200);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::ALL), -100);

        assert_eq!(contract.get_score_for_tricks(4, Vulnerability::NONE), -300);
        assert_eq!(contract.get_score_for_tricks(5, Vulnerability::NONE), -250);
        assert_eq!(contract.get_score_for_tricks(6, Vulnerability::NONE), -200);
        assert_eq!(contract.get_score_for_tricks(7, Vulnerability::NONE), -150);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::NONE), -100);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::NONE), -50);
    }

    #[test]
    fn score_doubled_down() {
        let bid = BidContract {
            suit: Strain::Spades,
            level: ContractLevel::Four,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(bid);

        assert_eq!(contract.get_score_for_tricks(4, Vulnerability::ALL), -1700);
        assert_eq!(contract.get_score_for_tricks(5, Vulnerability::ALL), -1400);
        assert_eq!(contract.get_score_for_tricks(6, Vulnerability::ALL), -1100);
        assert_eq!(contract.get_score_for_tricks(7, Vulnerability::ALL), -800);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::ALL), -500);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::ALL), -200);

        assert_eq!(contract.get_score_for_tricks(4, Vulnerability::NONE), -1400);
        assert_eq!(contract.get_score_for_tricks(5, Vulnerability::NONE), -1100);
        assert_eq!(contract.get_score_for_tricks(6, Vulnerability::NONE), -800);
        assert_eq!(contract.get_score_for_tricks(7, Vulnerability::NONE), -500);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::NONE), -300);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::NONE), -100);
    }

    #[test]
    fn score_redoubled_down() {
        let bid = BidContract {
            suit: Strain::Spades,
            level: ContractLevel::Four,
            modifier: ContractModifier::Redoubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(bid);

        assert_eq!(contract.get_score_for_tricks(4, Vulnerability::ALL), -3400);
        assert_eq!(contract.get_score_for_tricks(5, Vulnerability::ALL), -2800);
        assert_eq!(contract.get_score_for_tricks(6, Vulnerability::ALL), -2200);
        assert_eq!(contract.get_score_for_tricks(7, Vulnerability::ALL), -1600);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::ALL), -1000);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::ALL), -400);

        assert_eq!(contract.get_score_for_tricks(4, Vulnerability::NONE), -2800);
        assert_eq!(contract.get_score_for_tricks(5, Vulnerability::NONE), -2200);
        assert_eq!(contract.get_score_for_tricks(6, Vulnerability::NONE), -1600);
        assert_eq!(contract.get_score_for_tricks(7, Vulnerability::NONE), -1000);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::NONE), -600);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::NONE), -200);
    }
}
