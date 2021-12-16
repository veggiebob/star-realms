use crate::game::components::card::CardRef;
use crate::game::components::stack::{SimpleStack, Stack};
use crate::game::HandId;
use std::slice::Iter;

pub struct ActiveCard {
    pub id: HandId,
    pub card: CardRef,
    pub will_discard: bool,
    pub played_this_turn: bool
}

pub struct IdCardCollection {
    pub cards: SimpleStack<ActiveCard>
}

impl IdCardCollection {
    pub fn new(cards: SimpleStack<ActiveCard>) -> IdCardCollection {
        IdCardCollection {
            cards
        }
    }
    pub fn has(&self, id: HandId) -> bool {
        self.cards.iter().any(|c| c.id == id)
    }
}

impl Stack for IdCardCollection {
    type Item = ActiveCard;

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn remove(&mut self, index: usize) -> Option<Self::Item> {
        self.cards.remove(index)
    }

    fn get(&self, index: usize) -> Option<&Self::Item> {
        self.cards.get(index)
    }

    fn add(&mut self, item: Self::Item) {
        self.cards.add(item)
    }

    fn iter(&self) -> Iter<'_, Self::Item> {
        self.cards.iter()
    }
}