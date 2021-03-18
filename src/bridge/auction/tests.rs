use crate::bridge::auction::Error::InsufficientBid;
use crate::bridge::auction::{Auction, Bid::*, Error, StrainBid};
use crate::bridge::contract::Contract::PassedOut;
use crate::bridge::contract::{ContractLevel, Strain};
use crate::bridge::BridgeDirection;

#[test]
fn can_pass_out() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::N);
    auction.bid(Pass)?;
    assert_eq!(auction.is_completed(), false);
    auction.bid(Pass)?;
    auction.bid(Pass)?;
    assert_eq!(auction.is_completed(), false);
    assert!(auction.contract().is_none());

    auction.bid(Pass)?;
    assert_eq!(auction.is_completed(), true);
    assert_eq!(auction.contract(), Some(PassedOut));

    Ok(())
}

#[test]
fn can_bid_strain() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::S);
    auction.bid(Pass)?;
    auction.bid(RealBid(StrainBid {
        level: ContractLevel::One,
        strain: Strain::Diamonds,
    }))?;

    auction.bid(Pass)?;
    auction.bid(Pass)?;

    assert_eq!(auction.is_completed(), false);
    auction.bid(Pass)?;
    assert_eq!(auction.is_completed(), true);

    Ok(())
}

#[test]
fn disallow_insufficient() -> Result<(), Error> {
    let mut auction = Auction::new(BridgeDirection::S);
    auction.bid(Pass)?;
    auction.bid(RealBid(StrainBid {
        level: ContractLevel::One,
        strain: Strain::Diamonds,
    }))?;

    auction.bid(Pass)?;
    assert_ne!(auction.last_strain_bid, Pass); // make sure only strains are saved

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

    Ok(())
}

mod basic {
    use crate::bridge::auction::StrainBid;
    use crate::bridge::contract::{ContractLevel, Strain};

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
}
