use crate::bridge::auction::Error::InsufficientBid;
use crate::bridge::auction::{Auction, Bid::*, Error, StrainBid, DOUBLE, PASS, REDOUBLE};
use crate::bridge::contract::Contract::PassedOut;
use crate::bridge::contract::{BidContract, Contract, ContractLevel, Modifier, Strain};
use crate::bridge::BridgeDirection;

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
    assert_ne!(auction.last_strain_bid, PASS); // make sure only strains are saved

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

    if let RealBid(strain_bid) = auction.last_strain_bid {
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

#[test]
fn generate_contract() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::S);
    assert_eq!(auction.contract(), None);
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    assert_eq!(auction.contract(), None);
    auction.bid(PASS)?;
    assert_eq!(auction.contract(), Some(Contract::PassedOut));

    let mut auction = Auction::new(BridgeDirection::S);
    auction.bid(RealBid(StrainBid {
        level: ContractLevel::Two,
        strain: Strain::Spades,
    }))?;
    auction.bid(PASS)?;
    auction.bid(PASS)?;
    assert_eq!(auction.contract(), None);

    auction.bid(PASS)?;
    assert_eq!(
        auction.contract(),
        Some(Contract::BidContract(BidContract {
            strain: Strain::Spades,
            level: ContractLevel::Two,
            modifier: Modifier::Pass,
            declarer: BridgeDirection::S
        }))
    );

    Ok(())
}

mod basic {
    use crate::bridge::auction::StrainBid;
    use crate::bridge::contract::{ContractLevel, Strain};
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
