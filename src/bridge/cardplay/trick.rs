use crate::bridge::{turns, BridgeDirection};
use bridge_deck::Card;

#[derive(Eq, PartialEq, Debug)]
pub struct CompletedTrick {
    north: Card,
    east: Card,
    south: Card,
    west: Card,
}

impl CompletedTrick {
    fn new(lead: BridgeDirection, cards: Vec<Card>) -> Self {
        debug_assert_eq!(cards.len(), 4);
        let mut ordered_cards = turns(lead)
            .zip(cards.into_iter().cycle())
            .skip_while(|(pos, _)| *pos != BridgeDirection::N)
            .take(4)
            .map(|(_, card)| card);

        Self {
            north: ordered_cards.next().unwrap(),
            east: ordered_cards.next().unwrap(),
            south: ordered_cards.next().unwrap(),
            west: ordered_cards.next().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bridge::cardplay::trick::CompletedTrick;
    use crate::bridge::BridgeDirection;
    use bridge_deck::Card;

    #[test]
    fn new() {
        let trick = CompletedTrick::new(
            BridgeDirection::S,
            vec![Card::H2, Card::H3, Card::H4, Card::H5],
        );
        assert_eq!(
            trick,
            CompletedTrick {
                north: Card::H4,
                east: Card::H5,
                south: Card::H2,
                west: Card::H3,
            }
        );
    }
}
