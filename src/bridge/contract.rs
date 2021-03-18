use crate::bridge::contract::util::{over_score, trick_score};
use crate::bridge::{BridgeDirection, Vulnerability};
use std::cmp::max;

mod util;

#[derive(Debug, Eq, PartialEq)]
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
                    let overtricks = tricks_taken - tricks_needed;

                    let level_bid = actual_contract.level as usize;
                    let multiplier = match actual_contract.modifier {
                        ContractModifier::Passed => 1,
                        ContractModifier::Doubled => 2,
                        ContractModifier::Redoubled => 4,
                    };
                    let made_score = trick_score(actual_contract.strain, level_bid) * multiplier;
                    let over_score = over_score(actual_contract, overtricks, vul);
                    let is_game = made_score >= 100;
                    let made_bonus = if is_game {
                        if vul {
                            500
                        } else {
                            300
                        }
                    } else {
                        50
                    };
                    let insult_bonus = match actual_contract.modifier {
                        ContractModifier::Passed => 0,
                        ContractModifier::Doubled => 50,
                        ContractModifier::Redoubled => 100,
                    };
                    let slam_bonus = match level_bid {
                        1..=5 => 0,
                        6 => {
                            if vul {
                                750
                            } else {
                                500
                            }
                        }
                        7 => {
                            if vul {
                                1500
                            } else {
                                1000
                            }
                        }
                        _ => {
                            panic!("Invalid number of tricks")
                        }
                    };
                    made_score as i32 + over_score as i32 + made_bonus + insult_bonus + slam_bonus
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BidContract {
    strain: Strain,
    pub(crate) level: ContractLevel,
    modifier: ContractModifier,
    declarer: BridgeDirection,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Strain {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    NoTrump,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ContractLevel {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
            strain: Strain::Spades,
            level: ContractLevel::Four,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::S,
        };
    }

    #[test]
    fn score_undoubled_down() {
        let bid = BidContract {
            strain: Strain::Spades,
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
            strain: Strain::Spades,
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
            strain: Strain::Spades,
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

    #[test]
    fn partscores_made_undoubled() {
        let major_bid = BidContract {
            strain: Strain::Spades,
            level: ContractLevel::Two,
            modifier: ContractModifier::Passed,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(major_bid);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::NONE), 110);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::ALL), 140);

        let minor_bid = BidContract {
            strain: Strain::Diamonds,
            level: ContractLevel::One,
            modifier: ContractModifier::Passed,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(minor_bid);
        assert_eq!(contract.get_score_for_tricks(7, Vulnerability::ALL), 70);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 190);
    }

    #[test]
    fn games_undoubled() {
        let major_bid = BidContract {
            strain: Strain::Spades,
            level: ContractLevel::Four,
            modifier: ContractModifier::Passed,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(major_bid);
        assert_eq!(contract.get_score_for_tricks(10, Vulnerability::NONE), 420);
        assert_eq!(contract.get_score_for_tricks(11, Vulnerability::NONE), 450);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::NONE), 480);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::NONE), 510);
        assert_eq!(contract.get_score_for_tricks(10, Vulnerability::ALL), 620);
        assert_eq!(contract.get_score_for_tricks(11, Vulnerability::ALL), 650);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::ALL), 680);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 710);

        let nt_game = BidContract {
            strain: Strain::NoTrump,
            level: ContractLevel::Three,
            modifier: ContractModifier::Passed,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(nt_game);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::NONE), 400);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 720);

        let minor_bid = BidContract {
            strain: Strain::Diamonds,
            level: ContractLevel::Five,
            modifier: ContractModifier::Passed,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(minor_bid);
        assert_eq!(contract.get_score_for_tricks(11, Vulnerability::ALL), 600);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::NONE), 440);
    }

    #[test]
    fn games_doubled() {
        let major_bid = BidContract {
            strain: Strain::Spades,
            level: ContractLevel::Four,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(major_bid);
        assert_eq!(contract.get_score_for_tricks(10, Vulnerability::NONE), 590);
        assert_eq!(contract.get_score_for_tricks(11, Vulnerability::NONE), 690);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::NONE), 790);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::NONE), 890);
        assert_eq!(contract.get_score_for_tricks(10, Vulnerability::ALL), 790);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 1390);

        let nt_game = BidContract {
            strain: Strain::NoTrump,
            level: ContractLevel::Three,
            modifier: ContractModifier::Redoubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(nt_game);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::ALL), 1000);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::NONE), 1400);

        let higher_nt_game = BidContract {
            strain: Strain::NoTrump,
            level: ContractLevel::Five,
            modifier: ContractModifier::Redoubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(higher_nt_game);
        assert_eq!(contract.get_score_for_tricks(11, Vulnerability::ALL), 1240);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::NONE), 1240);
        let minor_game = BidContract {
            strain: Strain::Clubs,
            level: ContractLevel::Five,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(minor_game);
        assert_eq!(contract.get_score_for_tricks(11, Vulnerability::ALL), 750);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::NONE), 650);
    }

    #[test]
    fn doubled_partscores() {
        let minor_no_game = BidContract {
            strain: Strain::Clubs,
            level: ContractLevel::Two,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(minor_no_game);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::ALL), 180);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::ALL), 380);
        assert_eq!(contract.get_score_for_tricks(10, Vulnerability::ALL), 580);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 1180);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::NONE), 180);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::NONE), 280);

        let minor_redoubled_into_game = BidContract {
            strain: Strain::Clubs,
            level: ContractLevel::Two,
            modifier: ContractModifier::Redoubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(minor_redoubled_into_game);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::ALL), 760);
        assert_eq!(contract.get_score_for_tricks(9, Vulnerability::ALL), 1160);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 2760);
        assert_eq!(contract.get_score_for_tricks(10, Vulnerability::NONE), 960);

        let two_nt = BidContract {
            strain: Strain::NoTrump,
            level: ContractLevel::Two,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(two_nt);
        assert_eq!(contract.get_score_for_tricks(8, Vulnerability::NONE), 490);
    }

    #[test]
    fn slams() {
        let minor = BidContract {
            strain: Strain::Clubs,
            level: ContractLevel::Six,
            modifier: ContractModifier::Passed,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(minor);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::NONE), 920);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::ALL), 1370);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::NONE), 940);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 1390);

        let major_doubled = BidContract {
            strain: Strain::Hearts,
            level: ContractLevel::Six,
            modifier: ContractModifier::Doubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(major_doubled);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::NONE), 1210);
        assert_eq!(contract.get_score_for_tricks(12, Vulnerability::ALL), 1660);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::NONE), 1310);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 1860);

        let nt_grand_re = BidContract {
            strain: Strain::NoTrump,
            level: ContractLevel::Seven,
            modifier: ContractModifier::Redoubled,
            declarer: BridgeDirection::N,
        };
        let contract = Contract::BidContract(nt_grand_re);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::NONE), 2280);
        assert_eq!(contract.get_score_for_tricks(13, Vulnerability::ALL), 2980);
    }
}
