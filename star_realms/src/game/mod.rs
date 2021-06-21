extern crate rand;
extern crate regex;
use std::collections::{HashSet, HashMap};

use crate::game::components::card::{Base, Card};

use self::rand::Rng;
use std::ops::{Add, AddAssign};
use crate::game::components::{Coin, Authority, Combat};

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
    trade: Coin,
    authority: Authority,
    combat: Combat,
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

pub type ConfigError = &'static str;
pub type ActionFunc = Box<dyn FnMut(&mut GameState, u8) -> Option<ConfigError>>;

pub struct ActionMeta {
    description: &'static str,
    config_description: Option<HashMap<u8, &'static str>>,
}

pub type ConditionFunc = Box<dyn FnMut(&GameState) -> bool>;

pub fn get_condition(name: &String) -> Option<ConditionFunc> {
    todo!()
}

pub fn get_action(name: &String) -> Option<(ActionMeta, ActionFunc)> {
    // signal to be a good
    if name.starts_with("G") {
        return None
    }
    match name.as_str() {
        "test" => {
            Some(
                (
                    ActionMeta {
                        description: "test",
                        config_description: None
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

pub fn parse_good_action(good_str: &String) -> Option<ActionFunc> {
    // parse string???
    // I'm thinking something like "G(\d*):(\d*):(\d*)" for trade, authority, and combat, respectively
    todo!()
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
            combat: self.combat + rhs.combat
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