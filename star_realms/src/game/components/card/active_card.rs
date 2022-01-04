use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::slice::Iter;

use ansi_term::Color;

use crate::game::CardStack;
use crate::game::components::card::CardRef;
use crate::game::components::card::details::ExhaustionLevel;
use crate::game::components::card::in_game::ActivePlay;
use crate::game::components::stack::{SimpleStack, Stack};

/// briefly describes how a card has been brought into play
pub struct ActiveCard {
    pub card: CardRef,
    pub will_discard: bool,
    pub played_this_turn: bool,

    ///
    pub content: Option<Vec<ActivePlay>>
}

/// stack of active cards
// used to have ids attached to them, but not anymore
pub struct IdCardCollection {
    pub cards: SimpleStack<ActiveCard>
}

impl IdCardCollection {
    pub fn new(cards: SimpleStack<ActiveCard>) -> IdCardCollection {
        IdCardCollection {
            cards,
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
        let content = (&item.content).clone().map(|ps| ps.iter().map(ActivePlay::new).collect());
        let card = ActiveCard {
            card: item,
            will_discard: true, // ???
            played_this_turn: true, // ???
            content
        };
        self.add(card);
    }

    fn remove(&mut self, index: usize) -> Option<CardRef> {
        <IdCardCollection as Stack<ActiveCard>>::remove(self, index).map(|c| c.card)
    }
}

impl ActiveCard {

    /// if it's a valid index, then it is Some()
    ///     in that case, if it can't be used anymore, it will be None
    ///     otherwise, it will be Some(n) for a number of times more that it can be used
    pub fn exhaustion(&self, index: usize) -> Option<Option<ExhaustionLevel>> {
        self.content.as_ref().and_then(|c| c.get(index).map(|f| f.exhaustions_left()))
    }
}