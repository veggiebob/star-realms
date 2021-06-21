pub mod components;

extern crate rand;
use crate::game::components::card::{Card, Base};
use std::collections::HashSet;
use crate::game::components::faction::Faction::Mech;
use crate::game::ActionError::NoAction;
use self::rand::Rng;

type CardStack = Stack<Card>;

#[derive(Debug)]
pub struct Stack<T> {
    elements: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new (elements: Vec<T>) -> Stack<T> {
        Stack {
            elements
        }
    }
    pub fn empty () -> Stack<T> {
        Stack {
            elements: vec![]
        }
    }
    pub fn len (&self) -> usize {
        self.elements.len()
    }
    pub fn add(&mut self, element: T) {
        self.elements.push(element);
    }

    /// we say that the "top" card is the last index card
    pub fn draw (&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            Some(self.elements.remove(self.elements.len() - 1))
        }
    }
    pub fn shuffle (&mut self) {
        if self.len() < 2 {
            return
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
            let r = rng.gen_range(0..max_len-i);
            self.add(new_stack.elements.remove(r));
        }
    }
}

pub struct PlayerArea {
    discard: CardStack,
    deck: CardStack,
    hand: CardStack,
    bases: CardStack,
    scrapped: CardStack,
}

impl PlayerArea {
    fn draw (&mut self) -> Card {
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
    Player2
}
pub struct GameState {
    player1: PlayerArea,
    player2: PlayerArea,
    current_player: CurrentPlayer,
    trade_row: CardStack,
    explorers: u8,
    scrapped: CardStack,
    trade_row_stack: CardStack,
}

impl GameState {
    fn get_current_player (&self) -> &PlayerArea {
        match &self.current_player {
            Player1 => &self.player1,
            Player2 => &self.player2,
        }
    }
}

pub enum ActionError {
    NoAction,
    BadInput
}

type ActionFunc = Box<dyn FnMut(&mut GameState, u8)>;

fn get_action (name: &String) -> Result<ActionFunc, ActionError> {
    if name == "test" {
        Ok(Box::new(|game: &mut GameState, _: u8| {
            game.player1.hand.add(Card {
                name: String::from("bazinga"),
                base: Some(Base::Outpost(4)),
                synergizes_with: HashSet::new(),
                effects: HashSet::new(),
            })
        }))
    } else {
        Err(NoAction)
    }
}