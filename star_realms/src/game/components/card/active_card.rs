use crate::game::components::card::CardRef;
use crate::game::components::stack::{SimpleStack, Stack};
use crate::game::{CardStack};
use std::slice::Iter;
use std::collections::{HashSet, HashMap};
use std::rc::{Rc, Weak};
use ansi_term::Color;

/// briefly describes how a card has been brought into play
pub struct ActiveCard {
    pub card: CardRef,
    pub will_discard: bool,
    pub played_this_turn: bool
}

///
pub struct IdCardCollection {
    pub cards: SimpleStack<ActiveCard>
}

impl IdCardCollection {
    pub fn new(cards: SimpleStack<ActiveCard>) -> IdCardCollection {
        IdCardCollection {
            cards
        }
    }
}



impl Stack<ActiveCard> for IdCardCollection {
    fn len(&self) -> usize {
        self.cards.len()
    }

    fn get(&self, index: usize) -> Option<&ActiveCard> {
        self.cards.get(index)
    }

    fn iter(&self) -> Box<dyn Iterator<Item=&ActiveCard> + '_> {
        Box::new(self.cards.iter())
    }

    fn add(&mut self, item: ActiveCard) {
        self.cards.add(item);
    }

    fn remove(&mut self, index: usize) -> Option<ActiveCard> {
        self.cards.remove(index)
    }
}

impl Stack<CardRef> for IdCardCollection {
    fn len(&self) -> usize {
        <IdCardCollection as Stack<ActiveCard>>::len(self)
    }

    fn get(&self, index: usize) -> Option<&CardRef> {
        self.cards.get(index).map(|v| &v.card)
    }

    fn iter(&self) -> Box<dyn Iterator<Item=&CardRef> + '_> {
        Box::new(self.cards.iter().map(|c| &c.card))
    }

    fn add(&mut self, item: CardRef) {
        println!("{}", Color::Yellow.paint("Warning! Adding a card to an IdCardCollection without specifying properties of an ActiveCard!"));
        let card = ActiveCard {
            card: item,
            will_discard: true, // ???
            played_this_turn: true // ???
        };
        self.add(card);
    }

    fn remove(&mut self, index: usize) -> Option<CardRef> {
        <IdCardCollection as Stack<ActiveCard>>::remove(self, index).map(|c| c.card)
    }
}