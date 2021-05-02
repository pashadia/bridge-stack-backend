use bridge_deck::Cards;

pub struct BridgeBoard {
    pub north: Cards,
    pub east: Cards,
    pub south: Cards,
    pub west: Cards,
}

impl BridgeBoard {
    pub fn deal() -> Self {
        let mut full_deck = Cards::ALL;
        Self {
            north: full_deck.pick(13).expect("Should be able to get 13 cards"),
            east: full_deck.pick(13).expect("Should be able to get 13 cards"),
            south: full_deck.pick(13).expect("Should be able to get 13 cards"),
            west: full_deck.pick(13).expect("Should be able to get 13 cards"),
        }
    }

    pub fn is_valid(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use crate::bridge::board::BridgeBoard;

    #[test]
    fn all_cards_should_exist() {
        let board = BridgeBoard::deal();
        let cards = board
            .north
            .union(board.east)
            .union(board.south)
            .union(board.west);
        assert_eq!(cards.len(), 52)
    }
    #[test]
    fn correct_number_of_cards() {
        let board = BridgeBoard::deal();
        assert_eq!(board.north.len(), 13);
        assert_eq!(board.east.len(), 13);
        assert_eq!(board.south.len(), 13);
        assert_eq!(board.west.len(), 13);
    }
}
