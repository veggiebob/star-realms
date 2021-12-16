use crate::game::components::card::CardRef;
use crate::game::components::stack::{SimpleStack, Stack};
use crate::game::{HandId, CardStack};
use std::slice::Iter;
use std::collections::{HashSet, HashMap};
use std::rc::{Rc, Weak};

type ActiveCardRef = Rc<ActiveCard>;

/// briefly describes how a card has been brought into play
pub struct ActiveCard {
    pub id: HandId,
    pub card: CardRef,
    pub will_discard: bool,
    pub played_this_turn: bool
}

pub struct IdCardCollection {
    pub cards: SimpleStack<ActiveCard>,
    ids: HashMap<HandId, usize>,
    indices: HashMap<usize, HandId>
}

impl IdCardCollection {
    pub fn new(cards: CardStack, current_ids: &HashSet<HandId>) -> IdCardCollection {
        let mut new_cards = SimpleStack::empty();
        let mut ids = HashMap::new();
        let mut indices = HashMap::new();
        let mut id = 0 as HandId; // id
        let mut i = 0 as usize; // index
        while i < cards.len() {
            if !current_ids.contains(&id) {
                let card = ActiveCard {
                    id,
                    card: Rc::clone(cards.get(i).unwrap()),
                    will_discard: false,
                    played_this_turn: false
                };
                new_cards.add(card);
                indices.insert(i, id);
                ids.insert(id, i);
            }
            id += 1;
        }
        IdCardCollection {
            cards: new_cards,
            ids,
            indices
        }
    }
    pub fn get_ids(&self) -> HashSet<HandId> {
        self.ids.keys().map(Clone::clone).collect()
    }
    pub fn has(&self, id: HandId) -> bool {
        self.cards.iter().any(|c| c.id == id)
    }
}



impl Stack<ActiveCard> for IdCardCollection {
    fn len(&self) -> usize {
        self.cards.len()
    }

    fn get(&self, index: usize) -> Option<&ActiveCard> {
        self.cards.get(index)
    }

    fn iter(&self) -> Iter<'_, ActiveCard> {
        self.cards.iter()
    }

    fn add(&mut self, item: ActiveCard) {
        let index = self.cards.len() - 1;
        self.ids.insert(item.id.clone(), index);
        self.indices.insert(index, item.id.clone());
        self.cards.add(item);
    }

    fn remove(&mut self, index: usize) -> Option<ActiveCard> {
        match self.cards.remove(index) {
            Some(card) => {
                let id = self.indices.remove(&index).unwrap();
                self.ids.remove(&id).unwrap();
                Some(card)
            },
            None => None
        }
    }
}

impl Stack<CardRef> for IdCardCollection {
    fn len(&self) -> usize {
        <IdCardCollection as Stack<ActiveCard>>::len(self)
    }

    fn get(&self, index: usize) -> Option<&CardRef> {
        self.cards.get(index).map(|v| &v.card)
    }

    fn iter(&self) -> Iter<'_, CardRef> {
        todo!("I hope you never have to iterate over CardRefs in an IdCardCollection!")
        // self.cards.iter().into_iter().map(|c| Rc::clone(&c.card)).collect::<Iter<_>>()
    }

    fn add(&mut self, item: CardRef) {
         let new_id = {
             let mut id = 0;
             while self.ids.contains_key(&id) {
                 id += 1;
             }
             id
         };
        let card = ActiveCard {
            id: new_id, // todo: ???
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