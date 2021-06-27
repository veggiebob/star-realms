extern crate rand;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use components::stack::Stack;

use crate::game::card_library::CardLibrary;
use crate::game::components::{Authority, Coin, Combat};
use crate::game::components::card::{Card, CardStatus};
use crate::game::util::Failure;

use self::rand::Rng;
use crate::game::effects::{ConfigSupplier, get_condition, get_action};
use crate::game::util::Failure::{Succeed, Fail};

pub mod components;
pub mod card_library;
pub mod effects;
mod util;

type CardStack = Stack<Card>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Goods {
    pub(crate) trade: Coin,
    pub(crate) authority: Authority,
    pub(crate) combat: Combat,
}

pub struct PlayerArea {
    discard: CardStack,
    deck: CardStack,
    hand_id: HashMap<u32, (Card, CardStatus)>, // all cards in hand or in play (including bases)
    scrapped: CardStack,
    goods: Goods,
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

pub struct GameState {
    player1: PlayerArea,
    player2: PlayerArea,
    current_player: Player,
    trade_row: CardStack,
    explorers: u8,
    scrapped: CardStack,
    trade_row_stack: CardStack,
    card_library: Rc<CardLibrary>,
}

pub enum Feedback {
    Invalid(String),
    Info(String),
}

pub enum UserActionIntent<T> {
    Continue(T),
    Finish
}
pub trait UserActionSupplier {
    fn select_effect(&self, game: &GameState) -> UserActionIntent<(u32, (String, String))>;
    fn select_card(&self, game: &GameState, from_who: &RelativePlayer) -> u32;
    fn select_cards(&self, game: &GameState, from_who: &RelativePlayer) -> HashSet<u32>;
    fn on_feedback(&self, feedback: Feedback);
}

impl PlayerArea {
    pub fn new(scout: Card, viper: Card, first: bool) -> PlayerArea {
        let mut pa = PlayerArea {
            discard: CardStack::empty(),
            deck: CardStack::empty(),
            scrapped: CardStack::empty(),
            hand_id: HashMap::new(),
            goods: Goods {
                combat: 0,
                authority: 0,
                trade: 0
            }
        };
        for _ in 0..8 {
            pa.deck.add(scout.clone());
        }
        for _ in 0..2 {
            pa.deck.add(viper.clone());
        }
        pa.deck.shuffle();
        if first {
            // todo: hardcoded
            pa.draw_hand(3);
        } else {
            pa.draw_hand(5);
        }
        pa
    }
    pub fn get_card_in_hand(&self, id: &u32) -> Option<&(Card, CardStatus)> {
        self.hand_id.get(id)
    }
    pub fn get_card_in_hand_mut(&mut self, id: &u32) -> Option<&mut (Card, CardStatus)> {
        self.hand_id.get_mut(id)
    }
    pub fn draw_hand(&mut self, num_cards: u8) {
        let mut id_index = 0;
        for _ in 0..num_cards {
            let card = self.draw();
            while self.hand_id.contains_key(&id_index) {
                id_index += 1;
            }
            self.hand_id.insert(id_index, (card, CardStatus::new()));
        }
    }
    pub fn discard_hand(&mut self) {
        let keys_vec = {
            let mut tmp = vec![];
            let keys = self.hand_id.keys();
            for id in keys {
                tmp.push(*id);
            }
            tmp
        };
        for id in keys_vec {
            if let Failure::Fail(_) = self.discard_by_id(&id) {
                panic!("PlayerArea::discard_hand: id {} is not in hand_id somehow", id);
            }
        }
    }
    pub fn discard_by_id(&mut self, id: &u32) -> Failure<String> {
        match self.hand_id.remove(id) {
            Some((card, _)) => {
                self.discard.add(card);
                Failure::Succeed
            },
            None => Failure::Fail(format!("cannot discard card by id {}!", id))
        }
    }
    fn draw(&mut self) -> Card {
        if let Some(c) = self.deck.draw() {
            c
        } else {
            self.discard.shuffle();
            while let Some(c) = self.discard.draw() {
                self.deck.add(c);
            }
            self.draw() // this *should* never recurse infinitely because you start with 10 cards lol
        }
    }
}

impl GameState {
    /// panics if there is no scout or viper
    /// this is helpful https://www.starrealms.com/sets-and-expansions/
    fn new (card_library: Rc<CardLibrary>) -> GameState {
        let scout = card_library.get_scout().expect("card library needs a scout!");
        let viper = card_library.get_viper().expect("card library needs a viper!");
        let mut gs = GameState {
            player1: PlayerArea::new((*scout).clone(), (*viper).clone(), true),
            player2: PlayerArea::new((*scout).clone(), (*viper).clone(), false),
            current_player: Player::Player1,
            trade_row: CardStack::empty(),
            explorers: 10,
            scrapped: CardStack::empty(),
            trade_row_stack: {
                let mut stack = CardStack::new(card_library.get_new_trade_stack());
                stack.shuffle();
                stack
            },
            card_library: card_library.clone(),
        };
        // todo: number of cards in trade row hard-coded
        gs.fill_trade_row(5);
        gs
    }
    fn fill_trade_row(&mut self, num: usize) {
        let left = num - self.trade_row.len();
        for _ in 0..left {
            match self.trade_row_stack.draw() {
                None => break,
                Some(card) => self.trade_row.add(card)
            }
        }
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


    /// A Result::Err(Feedback::Invalid) represents an *internal failure* that should be
    ///     reported to the client, such as a bad config
    /// A Result::Err(Feedback::Info) represents an incomplete process that should be
    ///     reported to the client and the user.
    /// A Result::Ok(Option::Some) represents a successful operation that should be reported
    ///     to the user.
    /// A Result::Ok(Option::None) represents a successful operation that does not require comment.
    pub fn advance<T>(&mut self, client: T) -> Result<Option<String>, Feedback>
        where T: ConfigSupplier + UserActionSupplier {
        if let UserActionIntent::Continue((card_id, (cond_s, act_s)))
                // select to either exit, or continue with an effect
            = client.select_effect(self) {
            // then parse the condition and action of this effect
            let mut cond = get_condition(cond_s.clone())
                .expect(
                    format!(
                        "GameState.advance(): bad selection condition {}", &cond_s)
                        .as_str());
            let (action_meta, mut action_func) = get_action(&act_s)
                .expect(
                    format!("GameState.advance(): bad selection action {}", &act_s)
                        .as_str());
            // evaluate the condition
            if cond(self, card_id) {
                // if true, run the action
                match action_func(self,
                               match action_meta.config {
                                   Some(config) => client.get_config(self, &config),
                                   _ => 0,
                               }) {
                    // if the action fails, then a bad config was passed in.
                    // perhaps we can report these better
                    Fail(msg) => Err(Feedback::Invalid(
                        format!(
                            "Unable to complete action to {}. {}",
                            action_meta.description.clone(),
                            msg)
                    )),
                    // if it succeeds, make sure to consume the effect
                    Succeed => {
                        match self.get_current_player_mut().get_card_in_hand_mut(&card_id) {
                            Some((_, card_status)) => {
                                card_status.use_effect(&(cond_s, act_s));
                                Ok(None)
                            }
                            None => Err(
                                Feedback::Invalid(
                                    format!(
                                        "Card id {} is not one of {:?}",
                                        card_id,
                                        self.get_current_player().hand_id.keys())))
                        }
                    }
                }
            } else {
                // if the condition is not true, report the mistake to the client, or user?
                Err(Feedback::Info("This effect is not possible at the moment.".to_string()))
            }
        } else {
            // the client chooses to exit, and hand over the turn.
            // todo: warn them if they haven't completed all their effects
            // todo: automatically exit turn if all effects have been completed
            self.flip_turn();
            Ok(None)
        }
    }
}