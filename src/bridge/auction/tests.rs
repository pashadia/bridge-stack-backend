use crate::bridge::auction::{Auction, Bid::*};
use crate::bridge::contract::Contract::PassedOut;
use crate::bridge::BridgeDirection;

#[test]
fn can_pass_out() {
    let mut auction = Auction::new(BridgeDirection::N);
    auction.bid(Pass);
    assert_eq!(auction.is_completed(), false);
    auction.bid(Pass);
    auction.bid(Pass);
    assert_eq!(auction.is_completed(), false);
    assert!(auction.contract().is_none());

    auction.bid(Pass);
    assert_eq!(auction.is_completed(), true);
    assert_eq!(auction.contract(), Some(PassedOut));
}
