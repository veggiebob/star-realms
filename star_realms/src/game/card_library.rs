use crate::game::components::card::Card;
use std::collections::HashMap;
use crate::game::Stack;
use std::rc::Rc;

struct CardLibrary {
    all_cards: Vec<Rc<Card>>,
    card_map: HashMap<String, Rc<Card>>,
    trade_stack: Stack<Rc<Card>>
}

impl CardLibrary {
    fn new(trade_stack: Vec<Card>, misc_cards: Vec<Card>) -> Result<CardLibrary, String> {
        let misc_cards: Vec<Rc<Card>> = {
            let mut tmp = vec![];
            for card in misc_cards {
                tmp.push(Rc::new(card));
            }
            tmp
        };
        let trade_stack: Vec<Rc<Card>> = {
            let mut tmp = vec![];
            for card in trade_stack {
                tmp.push(Rc::new(card));
            }
            tmp
        };

        let mut map: HashMap<String, Rc<Card>> = HashMap::new();
        for card in misc_cards.iter() {
            map.insert(card.name.clone(), Rc::clone(card));
        }
        for card in trade_stack.iter() {
            map.insert(card.name.clone(), Rc::clone(card));
        }

        let mut ts: Stack<Rc<Card>> = Stack::empty();
        for card in trade_stack.iter() {
            ts.add(Rc::clone(card));
        }

        let mut all_cards: Vec<Rc<Card>> = vec![];
        for card in trade_stack {
            all_cards.push(card);
        }
        for card in misc_cards {
            all_cards.push(card);
        }

        let cl = CardLibrary {
            card_map: map,
            trade_stack: ts,
            all_cards
        };
        match cl.get_scout() {
            Some(_) => match cl.get_viper() {
                Some(_) => match cl.get_explorer() {
                    Some(_) => Ok(cl),
                    None => Err("No explorer available in card library!".to_string())
                }
                None => Err("No viper available in card library!".to_string())
            }
            None => Err("No scout available in card library!".to_string())
        }
    }

    fn get_card_by_name(&self, name: &str) -> Option<Rc<Card>> {
        match self.card_map.get(name) {
            Some(card) => Some(Rc::clone(card)),
            None => None
        }
    }

    fn get_scout(&self) -> Option<Rc<Card>> {
        self.get_card_by_name("scout")
    }
    fn get_viper(&self) -> Option<Rc<Card>> {
        self.get_card_by_name("viper")
    }
    fn get_explorer(&self) -> Option<Rc<Card>> {
        self.get_card_by_name("explorer")
    }
}

