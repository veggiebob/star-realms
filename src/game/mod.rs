
struct CardStack {
    cards: Vec<Card>,
}

impl CardStack {
    pub fn new (cards: Vec<Card>) -> CardStack {
        CardStack(cards)
    }
    pub fn len (&self) -> usize {
        self.cards.len()
    }
    pub fn add_card (&mut self, card: Card) {
        self.cards.push(card);
    }
    pub fn shuffle (&mut self) {
        panic!("no shuffling implemented yet");
    }
}

struct PlayerArea {
    discard: Cards,
    deck: Cards,
    hand: Cards,
    bases: Cards,
    scrapped: Cards
}