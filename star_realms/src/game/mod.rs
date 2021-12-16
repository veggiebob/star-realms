extern crate rand;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use components::card::active_card::IdCardCollection;
use components::stack::SimpleStack;

use crate::game::actions::client_comms::{Client, ClientActionOptionQuery, ClientQuery};
use crate::game::card_library::CardLibrary;
use crate::game::components::{Authority, Coin, Combat, Goods};
use crate::game::components::card::{Card, CardRef};
use crate::game::components::card::active_card::ActiveCard;
use crate::game::components::card::details::CardSource;
use crate::game::RelativePlayer::{Current, Opponent};
use crate::game::util::Failure;
use crate::game::util::Failure::{Fail, Succeed};
use crate::game::components::stack::Stack;

pub mod components;
pub mod card_library;
pub mod util;
pub mod actions;
pub mod requirements;

type CardStack = SimpleStack<CardRef>;
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
    pub trade_row: CardStack,
    pub explorers: u8,
    pub scrapped: CardStack,
    pub trade_row_stack: CardStack,
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

pub struct PlayerArea {
    hand: IdCardCollection,
    table: IdCardCollection,
    turn_data: TurnData,
    deck: CardStack,
    discard: CardStack,
    current_goods: Goods,
    ids: HashSet<HandId>
}

impl PlayerArea {
    pub fn new(scout: CardRef, viper: CardRef) -> PlayerArea {
        let mut pa = PlayerArea {
            hand: IdCardCollection::new(SimpleStack::empty()),
            table: IdCardCollection::new(SimpleStack::empty()),
            turn_data: TurnData {
                to_be_scrapped: HashSet::new(),
                to_be_discarded: HashSet::new(),
                played_this_turn: HashSet::new()
            },
            deck: {
                let mut stack = SimpleStack::empty();
                for _ in 0..8 {
                    stack.add(scout.clone());
                }
                for _ in 0..2 {
                    stack.add(viper.clone());
                }
                stack.shuffle();
                stack
            },
            discard: SimpleStack::empty(),
            current_goods: Goods::none(),
            ids: HashSet::new()
        };
        if let Fail(msg) = pa.draw_cards_into_hand(5) {
            println!("DEV WARNING: {}", msg);
        }
        pa
    }

    pub fn draw_card(&mut self) -> Option<CardRef> {
        if self.deck.len() + self.discard.len() == 0 {
            None
        } else {
            match self.deck.draw() {
                None => {
                    self.discard.shuffle();
                    self.discard.move_all_to(&mut self.deck);
                    self.draw_card()
                },
                x => x
            }
        }
    }

    pub fn draw_card_into_hand(&mut self) -> Failure<String> {
        match self.draw_card() {
            Some(card) => {
                self.hand.add(card);
                Succeed
            },
            None => Failure::Fail("No cards in deck nor discard".to_string())
        }
    }

    fn activate_card(&mut self,
                     card: CardRef,
                     will_discard: bool,
                     played_this_turn: bool
    ) -> ActiveCard {
        let mut id = 0;
        while self.ids.contains(&id) {
            id += 1;
        }
        self.ids.insert(id);
        ActiveCard {
            id,
            card,
            will_discard,
            played_this_turn,
        }
    }

    pub fn draw_cards_into_hand(&mut self, num: usize) -> Failure<String> {
        for i in 0..num {
            if let Fail(_) = self.draw_card_into_hand() {
                return Fail(format!("Failed to draw {} cards, drew {}.", num, i));
            }
        }
        Succeed
    }
}

impl GameState {
    /// ## Panic
    /// If there is no scout or viper (because CardLibrary can only be created using them)
    /// ## Other
    /// this is helpful https://www.starrealms.com/sets-and-expansions/
    pub fn new (card_library: Rc<CardLibrary>) -> GameState {
        let scout = card_library.get_scout().expect("card library needs a scout!");
        let viper = card_library.get_viper().expect("card library needs a viper!");
        let mut gs = GameState {
            player1: PlayerArea::new(Rc::clone(&scout), Rc::clone(&viper)),
            player2: PlayerArea::new(Rc::clone(&scout), Rc::clone(&viper)),
            current_player: Player::Player1,
            trade_row: SimpleStack::empty(),
            explorers: 10,
            scrapped: CardStack::empty(),
            trade_row_stack: {
                let mut stack = card_library.get_new_trade_stack();
                stack.shuffle();
                stack
            },
            card_library: Rc::clone(&card_library),
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
                Some(id) => self.trade_row.add(id)
            }
        }
    }

    /// ids: the indices of the cards to be removed
    pub fn remove_cards_from_trade_row(&mut self, ids: HashSet<u32>) -> HashSet<CardRef> {
        let mut ids: Vec<_> = ids.iter().collect();
        ids.sort();
        ids.reverse(); // remove them from biggest to smallest to prevent shifting
        let mut cards = HashSet::new();
        for i in ids {
            let card = self.trade_row.remove(*i as usize)
                .ok_or(format!("{} is not a valid index in the trade row", i)).unwrap();
            cards.insert(card.clone());
        }
        cards
    }

    pub fn get_stack_mut<S>(&mut self, card_source: CardSource) -> &mut S
        where S: Stack<Item=CardRef> {
        match card_source {
            CardSource::Deck(player) => match player {
                Current => &mut self.get_current_player_mut().deck,
                Opponent => &mut self.get_current_opponent_mut().deck
            },
            CardSource::Discard(player) => match player {
                Current => &mut self.get_current_player_mut().discard,
                Opponent => &mut self.get_current_opponent_mut().discard
            },
            CardSource::Hand(player) => match player {
                Current => &mut self.get_current_player_mut().hand.cards,
                Opponent => &mut self.get_current_opponent_mut().hand.cards
            },
            CardSource::TradeRow => &mut self.trade_row
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

    pub fn advance<T: Client>(&mut self, receivers: Vec<&T>) {
        let current = self.get_current_player_mut();

        // Turn layout:
        // (should have up to 5 cards in hand)
        // 1. take any of the actions on any of the cards, provided that it is able
        //  - keep a running sum of damage (combat)

        // 2. deal damage to the opponent
        // 3. discard all cards in hand
        //  - discard all cards scheduled to be discarded
        //  - scrap all cards scheduled to be scrapped
        current.hand.draw_to(&mut current.discard);

        // 4. draw 5 cards into hand
        current.draw_cards_into_hand(5);

    }

}