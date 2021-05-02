use crate::auction::Error::InsufficientBid;
use crate::auction::{Auction, Bid::*, Error, StrainBid, DOUBLE, PASS, REDOUBLE};
use crate::contract::Contract::PassedOut;
use crate::contract::{ContractLevel, Strain};
use crate::BridgeDirection;

#[test]
fn can_pass_out() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::N);
    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), false);
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), false);
    assert!(auction.contract().is_none());

    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), true);
    assert_eq!(auction.contract(), Some(PassedOut));

    Ok(())
}

#[test]
fn can_bid_strain() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::S);
    auction.bid(PASS)?;
    auction.bid(RealBid(StrainBid {
        level: ContractLevel::One,
        strain: Strain::Diamonds,
    }))?;

    auction.bid(PASS)?;
    auction.bid(PASS)?;

    assert_eq!(auction.is_completed(), false);
    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), true);

    Ok(())
}

#[test]
fn disallow_insufficient() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::S);
    auction.bid(PASS)?;
    auction.bid(RealBid(StrainBid {
        level: ContractLevel::One,
        strain: Strain::Diamonds,
    }))?;

    auction.bid(PASS)?;
    assert!(auction.last_strain_bid.is_some());

    auction.bid(RealBid(StrainBid {
        level: ContractLevel::One,
        strain: Strain::Spades,
    }))?;
    let insufficient = auction
        .bid(RealBid(StrainBid {
            level: ContractLevel::One,
            strain: Strain::Hearts,
        }))
        .unwrap_err();
    assert_eq!(insufficient, InsufficientBid);

    if let Some(strain_bid) = auction.last_strain_bid {
        assert_eq!(strain_bid.strain, Strain::Spades)
    } else {
        panic!("We should have something other than Pass by now")
    }

    auction.bid(PASS)?;
    auction
        .bid(RealBid(StrainBid {
            level: ContractLevel::One,
            strain: Strain::Spades,
        }))
        .unwrap_err();

    auction.bid(RealBid(StrainBid {
        level: ContractLevel::Three,
        strain: Strain::Spades,
    }))?;

    assert_eq!(
        auction
            .bid(RealBid(StrainBid {
                level: ContractLevel::Two,
                strain: Strain::Clubs,
            }))
            .unwrap_err(),
        Error::InsufficientBid
    );

    Ok(())
}

#[test]
fn doubles() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::S);
    auction.bid(RealBid(StrainBid {
        level: ContractLevel::One,
        strain: Strain::Diamonds,
    }))?;
    auction.bid(DOUBLE)?;
    assert_eq!(auction.bid(DOUBLE).unwrap_err(), Error::CantDouble);

    auction.bid(PASS)?;
    assert_eq!(auction.bid(DOUBLE).unwrap_err(), Error::CantDouble);

    assert_eq!(
        auction
            .bid(RealBid(StrainBid {
                level: ContractLevel::One,
                strain: Strain::Diamonds,
            }))
            .unwrap_err(),
        Error::InsufficientBid
    ); // Good to test this after double

    auction.bid(RealBid(StrainBid {
        level: ContractLevel::Three,
        strain: Strain::Diamonds,
    }))?;
    auction.bid(PASS)?;
    assert_eq!(auction.bid(DOUBLE).unwrap_err(), Error::CantDouble);
    auction.bid(PASS)?;
    auction.bid(DOUBLE)?; // This works, it's a reveil

    // Auction can't start with a double either
    let mut auction = Auction::new(BridgeDirection::S);
    assert_eq!(auction.bid(DOUBLE).unwrap_err(), Error::CantDouble);
    auction.bid(PASS)?;
    assert_eq!(auction.bid(DOUBLE).unwrap_err(), Error::CantDouble);

    Ok(())
}

#[test]
fn redoubles() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::E);
    assert_eq!(auction.bid(REDOUBLE).unwrap_err(), Error::CantRedouble);

    auction.bid(PASS)?;
    assert_eq!(auction.bid(REDOUBLE).unwrap_err(), Error::CantRedouble);

    auction.bid(RealBid(StrainBid {
        level: ContractLevel::Three,
        strain: Strain::Diamonds,
    }))?;
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    auction.bid(DOUBLE)?;
    auction.bid(PASS)?;

    // Partner doubled in reveil
    assert_eq!(auction.bid(REDOUBLE).unwrap_err(), Error::CantRedouble);

    auction.bid(PASS)?;
    auction.bid(REDOUBLE)?;

    // Fun ends after redouble
    assert_eq!(auction.bid(DOUBLE).unwrap_err(), Error::CantDouble);
    assert_eq!(auction.bid(REDOUBLE).unwrap_err(), Error::CantRedouble);

    // Auction can't start with a redouble either
    let mut auction = Auction::new(BridgeDirection::S);
    assert_eq!(auction.bid(REDOUBLE).unwrap_err(), Error::CantRedouble);
    auction.bid(PASS)?;
    assert_eq!(auction.bid(REDOUBLE).unwrap_err(), Error::CantRedouble);

    Ok(())
}

#[test]
fn auction_finished() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::W);
    assert_eq!(auction.is_completed(), false);
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), false);
    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), true);

    let mut auction = Auction::new(BridgeDirection::W);

    auction.bid(PASS)?;
    auction.bid(RealBid(StrainBid {
        level: ContractLevel::Three,
        strain: Strain::Diamonds,
    }))?;
    auction.bid(DOUBLE)?;
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), false);

    auction.bid(PASS)?;
    assert_eq!(auction.is_completed(), true);

    Ok(())
}

mod contract {
    use crate::auction::{Auction, Bid::*, Error, StrainBid, DOUBLE, PASS, REDOUBLE};
    use crate::contract::{BidContract, Contract, Modifier};
    use crate::BridgeDirection;
    use std::convert::{TryFrom, TryInto};

    #[test]
    fn passout() -> Result<(), Error> {
        let mut auction = Auction::new(BridgeDirection::S);
        assert_eq!(auction.contract(), None);
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        assert_eq!(auction.contract(), None);
        auction.bid(PASS)?;
        assert_eq!(auction.contract(), Some(Contract::PassedOut));

        Ok(())
    }

    #[test]
    fn basic_contract() -> Result<(), Error> {
        let mut auction = Auction::new(BridgeDirection::S);
        auction.bid(RealBid(StrainBid::try_from("2s").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        assert_eq!(auction.contract(), None);

        auction.bid(PASS)?;
        assert_eq!(
            auction.contract(),
            Some(Contract::BidContract(BidContract {
                contract: "2s".try_into().unwrap(),
                modifier: Modifier::Pass,
                declarer: BridgeDirection::S
            }))
        );

        Ok(())
    }
    #[test]
    fn doubled_contract() -> Result<(), Error> {
        let mut auction = Auction::new(BridgeDirection::S);
        auction.bid(RealBid(StrainBid::try_from("3n").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        auction.bid(DOUBLE)?;
        assert_eq!(auction.contract(), None);
        auction.bid(PASS)?;
        assert_eq!(auction.contract(), None);
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        assert_eq!(
            auction.contract(),
            Some(Contract::BidContract(BidContract {
                contract: "3N".try_into().unwrap(),
                modifier: Modifier::Double,
                declarer: BridgeDirection::S
            }))
        );

        Ok(())
    }

    #[test]
    fn redoubled_contract() -> Result<(), Error> {
        let mut auction = Auction::new(BridgeDirection::S);

        auction.bid(RealBid(StrainBid::try_from("1n").unwrap()))?;
        auction.bid(DOUBLE)?;
        auction.bid(PASS)?;
        auction.bid(REDOUBLE).unwrap_err(); // just checking
        auction.bid(RealBid(StrainBid::try_from("2n").unwrap()))?;

        auction.bid(DOUBLE)?;
        auction.bid(RealBid(StrainBid::try_from("3n").unwrap()))?;
        auction.bid(DOUBLE)?;
        auction.bid(REDOUBLE)?;

        assert_eq!(auction.contract(), None);
        auction.bid(PASS)?;
        assert_eq!(auction.contract(), None);
        auction.bid(PASS)?;
        auction.bid(PASS)?;

        assert_eq!(
            auction.contract(),
            Some(Contract::BidContract(BidContract {
                contract: "3N".try_into().unwrap(),
                modifier: Modifier::Redouble,
                declarer: BridgeDirection::E
            }))
        );

        Ok(())
    }

    #[test]
    fn declarer_is_first_to_name_contract() -> Result<(), Error> {
        let mut auction = Auction::new(BridgeDirection::S);
        auction.bid(RealBid(StrainBid::try_from("1nt").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(RealBid(StrainBid::try_from("3nt").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        assert_eq!(auction.contract(), None);

        auction.bid(PASS)?;
        assert_eq!(
            auction.contract(),
            Some(Contract::BidContract(BidContract {
                contract: "3N".try_into().unwrap(),
                modifier: Modifier::Pass,
                declarer: BridgeDirection::S
            }))
        );

        Ok(())
    }

    #[test]
    fn other_declarers() -> Result<(), Error> {
        let mut auction = Auction::new(BridgeDirection::W);
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        auction.bid(RealBid(StrainBid::try_from("4d").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;

        assert_eq!(
            auction.contract(),
            Some(Contract::BidContract(BidContract {
                contract: "4D".try_into().unwrap(),
                modifier: Modifier::Pass,
                declarer: BridgeDirection::E
            }))
        );

        let mut auction = Auction::new(BridgeDirection::W);
        auction.bid(RealBid(StrainBid::try_from("1d").unwrap()))?;
        auction.bid(RealBid(StrainBid::try_from("1s").unwrap()))?;
        auction.bid(RealBid(StrainBid::try_from("2d").unwrap()))?;
        auction.bid(RealBid(StrainBid::try_from("2h").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(RealBid(StrainBid::try_from("4h").unwrap()))?;
        // auction.bid(DOUBLE)?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        assert_eq!(
            auction.contract(),
            Some(Contract::BidContract(BidContract {
                contract: "4H".try_into().unwrap(),
                modifier: Modifier::Pass,
                declarer: BridgeDirection::S
            }))
        );

        Ok(())
    }

    #[test]
    fn declarer_did_actually_win_auction() -> Result<(), Error> {
        let mut auction = Auction::new(BridgeDirection::N);
        auction.bid(RealBid(StrainBid::try_from("1d").unwrap()))?;
        auction.bid(RealBid(StrainBid::try_from("2d").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(RealBid(StrainBid::try_from("3d").unwrap()))?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        auction.bid(PASS)?;
        assert_eq!(
            auction.contract(),
            Some(Contract::BidContract(BidContract {
                contract: "3d".try_into().unwrap(),
                modifier: Modifier::Pass,
                declarer: BridgeDirection::E
            }))
        );
        Ok(())
    }
}

mod basic {
    use crate::auction::StrainBid;
    use crate::contract::{ContractLevel, Strain};
    use std::convert::TryFrom;

    #[test]
    fn comparisons() {
        let two_clubs = StrainBid {
            level: ContractLevel::Two,
            strain: Strain::Clubs,
        };
        let three_spades = StrainBid {
            level: ContractLevel::Three,
            strain: Strain::Spades,
        };
        assert!(&two_clubs < &three_spades);
    }

    #[test]
    fn read_strain_bid() -> Result<(), &'static str> {
        assert_eq!(
            StrainBid::try_from("1c")?,
            StrainBid {
                level: ContractLevel::One,
                strain: Strain::Clubs
            }
        );
        assert_eq!(
            StrainBid::try_from("2c")?,
            StrainBid {
                level: ContractLevel::Two,
                strain: Strain::Clubs
            }
        );
        assert_eq!(
            StrainBid::try_from("4s")?,
            StrainBid {
                level: ContractLevel::Four,
                strain: Strain::Spades
            }
        );

        Ok(())
    }
}
