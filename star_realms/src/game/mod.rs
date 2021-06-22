extern crate rand;
extern crate regex;

use std::collections::{HashSet, HashMap};

use crate::game::components::card::{Base, Card};

use self::rand::Rng;
use std::ops::{Add, AddAssign};
use crate::game::components::{Coin, Authority, Combat};
use self::regex::Regex;
use crate::parse::parse_goods;

pub mod components;

type CardStack = Stack<Card>;

#[derive(Debug)]
pub struct Stack<T> {
    elements: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new(elements: Vec<T>) -> Stack<T> {
        Stack {
            elements
        }
    }
    pub fn empty() -> Stack<T> {
        Stack {
            elements: vec![]
        }
    }
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    pub fn add(&mut self, element: T) {
        self.elements.push(element);
    }

    /// we say that the "top" card is the last index card
    pub fn draw(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            Some(self.elements.remove(self.elements.len() - 1))
        }
    }
    pub fn shuffle(&mut self) {
        if self.len() < 2 {
            return;
        }
        let mut new_stack: Stack<T> = Stack::empty();
        let mut rng = rand::thread_rng();

        // move all the elements into a different stack
        let max_len = self.elements.len();
        for i in (0..max_len).rev() {
            new_stack.add(self.elements.remove(i));
        }

        // replace them randomly
        for i in 0..max_len {
            let r = rng.gen_range(0..max_len - i);
            self.add(new_stack.elements.remove(r));
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Goods {
    pub(crate) trade: Coin,
    pub(crate) authority: Authority,
    pub(crate) combat: Combat,
}

pub struct PlayerArea {
    discard: CardStack,
    deck: CardStack,
    hand: CardStack,
    bases: CardStack,
    scrapped: CardStack,
    goods: Goods,
}

impl PlayerArea {
    fn draw(&mut self) -> Card {
        if let Some(c) = self.deck.draw() {
            c
        } else {
            self.discard.shuffle();
            while let Some(c) = self.discard.draw() {
                self.deck.add(c);
            }
            self.draw()
        }
    }
}

pub enum CurrentPlayer {
    Player1,
    Player2,
}

pub struct GameState {
    player1: PlayerArea,
    player2: PlayerArea,
    current_player: CurrentPlayer,
    trade_row: CardStack,
    explorers: u8,
    scrapped: CardStack,
    trade_row_stack: CardStack,
    goods: Goods,
}

impl GameState {
    fn get_current_player(&self) -> &PlayerArea {
        match &self.current_player {
            Player1 => &self.player1,
            Player2 => &self.player2,
        }
    }
    fn get_current_player_mut(&mut self) -> &mut PlayerArea {
        match &self.current_player {
            Player1 => &mut self.player1,
            Player2 => &mut self.player2,
        }
    }
    fn turn_is_player1(&self) -> bool {
        match &self.current_player {
            Player1 => true,
            Player2 => false
        }
    }
    fn turn_is_player2(&self) -> bool {
        !self.turn_is_player1()
    }
}

/// Effects!

pub type ConfigError = &'static str;
pub type ActionFunc = Box<dyn FnMut(&mut GameState, u8) -> Option<ConfigError>>;

pub struct ActionMeta {
    description: &'static str,
    config_description: Option<HashMap<u8, &'static str>>,
}

pub type ConditionFunc = Box<dyn FnMut(&GameState) -> bool>;

pub fn validate_condition(name: &str) -> bool {
    get_condition(name).is_some()
}

pub fn validate_action(name: &str) -> bool {
    get_action(name).is_some()
}

pub fn validate_effect((cond, act): (&str, &str)) -> bool {
    validate_condition(cond) && validate_action(act)
}

pub fn validate_card(card: &Card) -> bool {
    card.effects.iter().all(|(l,r)| validate_effect((l.as_str(), r.as_str())))
}

pub fn get_condition(name: &str) -> Option<ConditionFunc> {
    match name {
        "any" => Some(Box::new(|_| true)),
        _ => None
    }
}

pub fn get_action(name: &str) -> Option<(ActionMeta, ActionFunc)> {
    // signal to be a good
    if name.starts_with("G") {
        return if let Some(goods) = parse_goods(name) {
            let action = get_good_action(goods);
            Some(
                (
                    ActionMeta { description: "gives some amount of trade, authority, and combat", config_description: None },
                    action
                )
            )
        } else {
            None
        }
    }
    match name {
        "test" => {
            Some(
                (
                    ActionMeta {
                        description: "test",
                        config_description: None,
                    },
                    Box::new(|game: &mut GameState, _: u8| {
                        game.player1.hand.add(Card {
                            name: String::from("bazinga"),
                            base: Some(Base::Outpost(4)),
                            synergizes_with: HashSet::new(),
                            effects: HashSet::new(),
                        });
                        None
                    })
                ))
        }
        _ => None
    }
}

pub fn get_good_action(goods: Goods) -> ActionFunc {
    Box::new(move |game: &mut GameState, _: u8| {
        game.get_current_player_mut().goods += goods;
        None
    })
}

impl ActionMeta {
    pub fn get_all_configs(&self) -> HashSet<&u8> {
        match &self.config_description {
            None => HashSet::new(),
            Some(cs) => {
                let mut h = HashSet::new();
                for (c, _) in cs {
                    h.insert(c);
                }
                h
            }
        }
    }
    pub fn no_config(&self) -> bool {
        self.config_description.is_none()
    }
}

impl Add for Goods {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Goods {
            trade: self.trade + rhs.trade,
            authority: self.authority + rhs.authority,
            combat: self.combat + rhs.combat,
        }
    }
}

impl AddAssign for Goods {
    fn add_assign(&mut self, rhs: Self) {
        self.trade += rhs.trade;
        self.authority += rhs.authority;
        self.combat += rhs.combat;
    }
}