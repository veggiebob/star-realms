extern crate rand;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::ops::{Add, AddAssign};

use crate::game::components::{Authority, Coin, Combat};
use crate::game::components::card::{Base, Card, CardStatus};
use crate::game::Player::Player1;
use crate::parse::parse_goods;

use self::rand::Rng;
use crate::game::util::Failure;
use crate::game::util::Failure::Success;
use crate::game::RelativePlayer::Opponent;
use crate::game::card_library::CardLibrary;
use std::rc::Rc;

pub mod components;
pub mod card_library;
mod util;

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

    /// draw a card from self and place it in other
    pub fn draw_to(&mut self, other: &mut Stack<T>) {
        match self.draw() {
            None => (),
            Some(card) => other.add(card),
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
    hand_id: HashMap<u32, (Card, CardStatus)>, // all cards in hand or in play (including bases)
    scrapped: CardStack,
    goods: Goods,
}

pub enum Player {
    Player1,
    Player2,
}

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

impl PlayerArea {
    pub fn new(scout: Card, viper: Card) -> PlayerArea {
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
        pa
    }
    pub fn get_card_in_hand(&self, id: &u32) -> Option<&(Card, CardStatus)> {
        self.hand_id.get(id)
    }
    pub fn draw_hand(&mut self) {
        let mut id_index = 0;
        for _ in 0..5 {
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
            if let Failure::Failure(_) = self.discard_by_id(&id) {
                panic!("PlayerArea::discard_hand: id {} is not in hand_id somehow", id);
            }
        }
    }
    pub fn discard_by_id(&mut self, id: &u32) -> Failure<String> {
        match self.hand_id.remove(id) {
            Some((card, _)) => {
                self.discard.add(card);
                Failure::Success
            },
            None => Failure::Failure(format!("cannot discard card by id {}!", id))
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
            player1: PlayerArea::new((*scout).clone(), (*viper).clone()),
            player2: PlayerArea::new((*scout).clone(), (*viper).clone()),
            current_player: Player1,
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
        gs.fill_trade_row();
        gs
    }
    fn fill_trade_row(&mut self) {
        // todo: number of cards in trade row hard-coded
        let left = 5 - self.trade_row.len();
        for _ in 0..left {
            match self.trade_row_stack.draw() {
                None => break,
                Some(card) => self.trade_row.add(card)
            }
        }
    }
    fn get_current_player(&self) -> &PlayerArea {
        match &self.current_player {
            Player::Player1 => &self.player1,
            Player::Player2 => &self.player2,
        }
    }
    fn get_current_player_mut(&mut self) -> &mut PlayerArea {
        match &self.current_player {
            Player1 => &mut self.player1,
            Player2 => &mut self.player2,
        }
    }
    fn get_current_opponent(&self) -> &PlayerArea {
        match &self.current_player {
            Player1 => &self.player2,
            Player2 => &self.player1
        }
    }
    fn get_current_opponent_mut(&mut self) -> &mut PlayerArea {
        match &self.current_player {
            Player1 => &mut self.player2,
            Player2 => &mut self.player1
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

pub type ConfigError = String;
pub type ActionFunc = Box<dyn FnMut(&mut GameState, u32) -> Failure<ConfigError>>;

pub struct ActionMeta {
    /// description of the action, (probably?) user-friendly
    description: String,
    config: Option<Config>
}
pub struct Config {
    /// dev-friendly description for each of the config values
    description: Box<dyn Fn(u32) -> String>,
    /// enum that shows how to get the config value (u32)
    config_method: ActionConfigMethod
}
//todo: are there any instances where a Range or Set would be used, and need to specify which
// player picks the config? If so, there should be a "by" player abstracted into Config as
// a sibling to ActionConfigMethod
pub enum ActionConfigMethod {
    /// low: u32, high: u32
    /// config should be a number in the range (low..high)
    Range(u32, u32),
    /// set: contains all the id's that can be used
    Set(HashSet<u32>), // in this set of numbers
    /// num: u32, by: Player, from: Player
    /// num = number of cards to pick
    /// by = player that is picking the cards
    /// from = player that is having cards be picked from
    /// config should be a bitwise-encoded number representing the cards that can be selected
    PickHandCards(u32, RelativePlayer, RelativePlayer),
    /// config should be the id of the card that can be picked
    /// by: Player, from: player
    /// by = player that is picking the cards
    /// from = player that is having cards be picked from
    PickHandCard(RelativePlayer, RelativePlayer),
    PickTradeRowCards(u32, RelativePlayer)
}

pub type ConditionFunc = Box<dyn FnMut(&GameState, u32) -> bool>;

pub fn validate_condition(name: &str) -> Option<String> {
    match get_condition(name) {
        Some(_) => None,
        None => Some(format!("Invalid condition: {}", name))
    }
}

pub fn validate_action(name: &str) -> Option<String> {
    match get_action(name) {
        Some(_) => None,
        None => Some(format!("Invalid action: {}", name))
    }
}

pub fn validate_effect((cond, act): (&str, &str)) -> Option<String> {
    match validate_condition(cond) {
        None => match validate_action(act) {
            None => None,
            x => x
        },
        x => x,
    }
}

/// None -> valid
/// String -> invalid, with reason
pub fn validate_card_effects(card: &Card) -> Option<String> {
    for (l, r) in card.effects.iter() {
        if let Some(e) = validate_effect((l.as_str(), r.as_str())) {
            return Some(e)
        }
    }
    None
}

pub fn assert_validate_card_effects(card: &Card) {
    if let Some(e) = validate_card_effects(&card) {
        panic!("{} was not a valid card because '{}': {:?}", card.name, e, card);
    }
}

pub fn get_condition(name: &str) -> Option<ConditionFunc> {
    match name {
        "any" => Some(Box::new(|_, _| true)),
        "trash" => Some(Box::new(
            |game, id| {
                game.get_current_player().hand_id.get(&id)
                    .expect("trash condition: bad id supplied")
                    .1.scrapped
            }
        )),
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
                    ActionMeta {
                        description: "gives some amount of trade, authority, and combat".to_string(),
                        config: None
                    },
                    action
                )
            )
        } else {
            None
        }
    }
    match name {
        "test" => Some(
                (
                    ActionMeta {
                        description: "test".to_string(),
                        config: None,
                    },
                    Box::new(|game: &mut GameState, _| {
                        game.player1.discard.add(Card {
                            cost: 255,
                            name: String::from("bazinga"),
                            base: Some(Base::Outpost(4)),
                            synergizes_with: HashSet::new(),
                            effects: HashSet::new(),
                        });
                        Success
                    })
                )
        ),
        "discard" => Some(
            (
                ActionMeta {
                    description: "opponent discards a card".to_string(),
                    config: Some(Config {
                        description: Box::new(|_| "hand id of card to be discarded".to_string()),
                        config_method: ActionConfigMethod::PickHandCard(Opponent, Opponent)
                    })
                },
                Box::new(|game: &mut GameState, cfg| {
                    let opponent = game.get_current_opponent_mut();
                    match opponent.hand_id.get(&cfg) {
                        None => Failure::Failure(format!("No card with id {}", &cfg)),
                        Some((_, card_status)) => if card_status.in_play {
                            Failure::Failure(
                                format!("Card is in play, player must discard from hand \
                                that has not been revealed"))
                        } else if let Failure::Failure(msg) = opponent.discard_by_id(&cfg) {
                            Failure::Failure(format!("unable to discard hand id {} in opponents hand: {}", &cfg, msg))
                        } else {
                            Success
                        }
                    }
                })
            )
        ),
        "destroy target base" => Some(
            (
                ActionMeta {
                    description: "destroy any of the opponents bases".to_string(),
                    config: Some(Config {
                        description: Box::new(|_| "hand id of the base to be destroyed".to_string()),
                        config_method: ActionConfigMethod::PickHandCard(RelativePlayer::Current, RelativePlayer::Opponent)
                    }),
                },
                Box::new(|game: &mut GameState, cfg| {
                    let opponent = game.get_current_opponent_mut();
                    match opponent.hand_id.get(&cfg) {
                        None => Failure::Failure(format!("No card with id {}", &cfg)),
                        Some((card, card_status)) => {
                            if !&card_status.in_play {
                                Failure::Failure(format!("Card {} must be in play!", &card.name))
                            } else if let None = &card.base {
                                Failure::Failure(format!("Card {} is not a base!", &card.name))
                            } else {
                                match opponent.discard_by_id(&cfg) {
                                    Success => Success,
                                    Failure::Failure(msg) => Failure::Failure(format!("Unable to discard this card because: {}", msg))
                                }
                            }
                        }
                    }
                })
            )
        ),
        _ => None
    }
}

pub fn get_good_action(goods: Goods) -> ActionFunc {
    Box::new(move |game: &mut GameState, _| {
        game.get_current_player_mut().goods += goods;
        Success
    })
}

impl ActionMeta {
    pub fn no_config(&self) -> bool {
        self.config.is_none()
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
