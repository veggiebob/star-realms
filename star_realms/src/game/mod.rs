extern crate rand;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use components::stack::Stack;

use crate::game::card_library::CardLibrary;
use crate::game::components::{Authority, Coin, Combat};
use crate::game::components::card::{Card, CardRef};
use crate::game::util::Failure;
use crate::game::util::Failure::{Fail, Succeed};

pub mod components;
pub mod card_library;
pub mod util;
pub mod actions;
pub mod requirements;

type CardStack = Stack<Card>;
pub type HandId = u32;

#[derive(Debug)]
pub struct TurnData {
    to_be_scrapped: HashSet<HandId>,
    to_be_discarded: HashSet<HandId>,
    played_this_turn: HashSet<HandId>
}

impl TurnData {
    pub fn new() -> TurnData {
        TurnData {
            to_be_scrapped: HashSet::new(),
            to_be_discarded: HashSet::new(),
            played_this_turn: HashSet::new()
        }
    }
    pub fn reset(&mut self)  {
        self.to_be_discarded = HashSet::new();
        self.to_be_scrapped = HashSet::new();
        self.played_this_turn = HashSet::new();
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Player {
    Player1,
    Player2,
}
impl Player {
    pub fn reverse(&self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RelativePlayer {
    Current,
    Opponent
}

impl RelativePlayer {
    pub fn to_string(&self) -> String {
        match self {
            RelativePlayer::Current => "current".to_string(),
            _ => "opponent".to_string()
        }
    }
}

pub struct GameState {
    player1: PlayerArea,
    player2: PlayerArea,
    current_player: Player,
    pub trade_row: Stack<u32>,
    pub explorers: u8,
    pub scrapped: CardStack,
    pub trade_row_stack: Stack<u32>,
    pub card_library: Rc<CardLibrary>,
}

pub enum Feedback {
    Invalid(String),
    Info(String),
}

pub enum UserActionIntent<T> {
    Continue(T),
    Cancel
}

pub enum AbstractPlayerAction {
    CardEffects,
    TradeRow,
    TrashCard,
    EndTurn,
}

pub trait UserActionSupplier {

    fn choose_abstract_action(&self, game: &GameState) -> AbstractPlayerAction;

    fn select_effect(&self, game: &GameState) -> UserActionIntent<(HandId, (String, String))>;

    /// return 0 to attempt to buy an explorer
    fn select_trade_row_card(&self, game: &GameState) -> UserActionIntent<HandId>;

    fn on_feedback(&self, feedback: Feedback);
}

pub struct PlayerArea {

}

impl PlayerArea {
    pub fn new(scout: CardRef, viper: CardRef) -> PlayerArea {
        todo!()
    }
}

impl GameState {
    /// panics if there is no scout or viper
    /// this is helpful https://www.starrealms.com/sets-and-expansions/
    pub fn new (card_library: Rc<CardLibrary>) -> GameState {
        let scout = card_library.get_scout().expect("card library needs a scout!");
        let viper = card_library.get_viper().expect("card library needs a viper!");
        let mut gs = GameState {
            player1: PlayerArea::new(Rc::clone(&scout), Rc::clone(&viper)),
            player2: PlayerArea::new(Rc::clone(&scout), Rc::clone(&viper)),
            current_player: Player::Player1,
            trade_row: Stack::empty(),
            explorers: 10,
            scrapped: CardStack::empty(),
            trade_row_stack: {
                let mut stack = Stack::new(card_library.get_new_trade_stack());
                stack.shuffle();
                stack
            },
            card_library: Rc::clone(&card_library),
        };
        // todo: number of cards in trade row hard-coded
        gs.fill_trade_row(5);
        gs
    }

    pub fn from_config(config_folder: &str) -> Result<GameState, String> {
        let cl = CardLibrary::from_config(config_folder)?;
        Ok(GameState::new(Rc::new(cl)))
    }

    fn fill_trade_row(&mut self, num: usize) {
        let left = num - self.trade_row.len();
        for _ in 0..left {
            match self.trade_row_stack.draw() {
                None => break,
                Some(id) => self.trade_row.add(id)
            }
        }
    }

    pub fn unpack_multi_trade_row_card_selection(&self, bits: &u32) -> HashSet<u32> {
        let mut ids = HashSet::new();
        let num_cards = self.trade_row.len();
        for i in 0..num_cards {
            if ((1<<i) & bits) > 0 {
                ids.insert(i as u32);
            }
        }
        ids
    }

    /// ids: the indices of the cards to be removed
    pub fn remove_cards_from_trade_row(&mut self, ids: HashSet<u32>) -> HashSet<CardRef> {
        let mut ids: Vec<_> = ids.iter().collect();
        ids.sort();
        ids.reverse(); // remove them from biggest to smallest to prevent shifting
        let mut cards = HashSet::new();
        for i in ids {
            let id = self.trade_row.remove(*i as usize)
                .ok_or(format!("{} is not a valid index in the trade row", i)).unwrap();
            cards.insert(Rc::clone(&self.card_library.get_card_by_id(&id).unwrap()));
        }
        cards
    }

    fn flip_turn(&mut self) {
        self.current_player = match self.current_player {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1
        }
    }
    pub fn resolve_relative(&self, relative_player: &RelativePlayer) -> Player {
        match relative_player {
            RelativePlayer::Current => self.current_player.clone(),
            RelativePlayer::Opponent => self.current_player.reverse()
        }
    }
    pub fn resolve_relative_player(&self, relative_player: &RelativePlayer) -> &PlayerArea {
        match relative_player {
            RelativePlayer::Current => self.get_current_player(),
            RelativePlayer::Opponent => self.get_current_opponent()
        }
    }

    pub fn resolve_relative_player_mut(&mut self, relative_player: &RelativePlayer) -> &mut PlayerArea {
        match relative_player {
            RelativePlayer::Current => self.get_current_player_mut(),
            RelativePlayer::Opponent => self.get_current_opponent_mut()
        }
    }
    pub fn get_current_player(&self) -> &PlayerArea {
        match &self.current_player {
            Player::Player1 => &self.player1,
            Player::Player2 => &self.player2,
        }
    }
    pub fn get_current_player_mut(&mut self) -> &mut PlayerArea {
        match &self.current_player {
            Player::Player1 => &mut self.player1,
            Player::Player2 => &mut self.player2,
        }
    }
    pub fn get_current_opponent(&self) -> &PlayerArea {
        match &self.current_player {
            Player::Player1 => &self.player2,
            Player::Player2 => &self.player1
        }
    }
    pub fn get_current_opponent_mut(&mut self) -> &mut PlayerArea {
        match &self.current_player {
            Player::Player1 => &mut self.player2,
            Player::Player2 => &mut self.player1
        }
    }
    pub fn turn_is_player1(&self) -> bool {
        match &self.current_player {
            Player::Player1 => true,
            Player::Player2 => false
        }
    }
    pub fn turn_is_player2(&self) -> bool {
        !self.turn_is_player1()
    }
}