use crate::game::components::card::{Card, CardRef};
use std::collections::HashMap;
use crate::game::components::stack::{SimpleStack, Stack};
use std::rc::Rc;
use crate::parse::parse_file;
use crate::game::CardStack;

pub struct CardLibrary {
    all_cards: Vec<CardRef>,
    id_map: HashMap<u32, CardRef>,
    id_lookup: HashMap<String, u32>,
    trade_stack: SimpleStack<u32>,
}

impl CardLibrary {
    pub fn from_config(config_folder: &str) -> Result<CardLibrary, String> {
        let trade_cards = parse_file(
            format!("{}/trade_cards.yaml", config_folder)
        )?;
        let misc_cards = parse_file(
            format!("{}/misc_cards.yaml", config_folder)
        )?;
        CardLibrary::new(trade_cards, misc_cards)
    }

    pub fn new(trade_stack: Vec<Card>, misc_cards: Vec<Card>) -> Result<CardLibrary, String> {
        let misc_cards = {
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

        let all_cards: Vec<Rc<Card>> = {
            let mut ac = vec![];
            for card in trade_stack.iter() {
                ac.push(Rc::clone(card));
            }
            for card in misc_cards.iter() {
                ac.push(Rc::clone(card));
            }
            ac
        };

        let (id_map, id_lookup) = {
            let mut id_map = HashMap::new();
            let mut id_lookup = HashMap::new();
            for (i, card) in all_cards.iter().enumerate() {
                id_map.insert(i as u32, Rc::clone(card));
                id_lookup.insert(card.name.clone(), i as u32);
            }
            (id_map, id_lookup)
        };

        let ts: SimpleStack<u32> = {
            let mut tmp = SimpleStack::empty();
            for card in trade_stack.iter() {
                tmp.add(id_lookup.get(&card.name).unwrap().clone());
            }
            tmp
        };

        let cl = CardLibrary {
            id_map,
            id_lookup,
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

    pub fn get_new_trade_stack(&self) -> CardStack {
        SimpleStack::new(
            self.trade_stack.elements
                .clone().into_iter()
                .map(|x| self.id_map.get(&x).unwrap().clone())
                .collect())
    }

    pub fn get_card_by_name(&self, name: &str) -> Option<Rc<Card>> {
        match self.id_lookup.get(name) {
            Some(id) => Some(self.get_card_by_id(id).unwrap()), // guarantee it, or else!
            None => None
        }
    }

    /// force-get a card
    pub fn as_card(&self, id: &u32) -> Rc<Card> {
        self.get_card_by_id(id).unwrap()
    }

    /// request a card politely
    pub fn get_card_by_id(&self, id: &u32) -> Option<Rc<Card>> {
        match self.id_map.get(id) {
            Some(c) => Some(Rc::clone(c)),
            None => None
        }
    }

    pub fn get_new_card_by_id(&self, id: &u32) -> Option<CardRef> {
        match self.get_card_by_id(id) {
            Some(c) => Some(Rc::clone(&c)),
            None => None
        }
    }

    pub fn get_card_id(&self, name: &String) -> Option<&u32> {
        self.id_lookup.get(name)
    }

    pub fn get_scout(&self) -> Option<Rc<Card>> {
        self.get_card_by_name("scout")
    }

    pub fn get_viper(&self) -> Option<Rc<Card>> {
        self.get_card_by_name("viper")
    }

    pub fn get_explorer(&self) -> Option<Rc<Card>> {
        self.get_card_by_name("explorer")
    }
}

