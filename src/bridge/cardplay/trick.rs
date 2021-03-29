use crate::bridge::{turns, BridgeDirection};
use cardpack::Card;

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
    use cardpack::{Card, FIVE, FOUR, HEARTS, THREE, TWO};

    #[test]
    fn new() {
        let trick = CompletedTrick::new(
            BridgeDirection::S,
            vec![
                Card::new(TWO, HEARTS),
                Card::new(THREE, HEARTS),
                Card::new(FOUR, HEARTS),
                Card::new(FIVE, HEARTS),
            ],
        );
        assert_eq!(
            trick,
            CompletedTrick {
                north: Card::new(FOUR, HEARTS),
                east: Card::new(FIVE, HEARTS),
                south: Card::new(TWO, HEARTS),
                west: Card::new(THREE, HEARTS),
            }
        );
    }
}
