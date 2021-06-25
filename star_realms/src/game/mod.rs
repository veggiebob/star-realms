extern crate rand;
extern crate regex;

use std::collections::{HashSet, HashMap};

use crate::game::components::card::{Base, Card, CardStatus};

use self::rand::Rng;
use std::ops::{Add, AddAssign};
use crate::game::components::{Coin, Authority, Combat};
use self::regex::Regex;
use crate::parse::parse_goods;
use std::iter::Map;
use crate::game::CurrentPlayer::Player1;

pub mod components;
pub mod card_library;

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
    hand_id: HashMap<u8, (Card, CardStatus)>,
    bases: CardStack,
    scrapped: CardStack,
    goods: Goods,
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
}

impl PlayerArea {
    pub fn new(scout: &Card, viper: &Card) -> PlayerArea {
        let mut pa = PlayerArea {
            discard: CardStack::empty(),
            deck: CardStack::empty(),
            bases: CardStack::empty(),
            scrapped: CardStack::empty(),
            hand_id: HashMap::new(),
            goods: Goods {
                combat: 0,
                authority: 0,
                trade: 0
            }
        };
        for i in 0..8 {
            pa.deck.add(scout.clone());
        }
        for i in 0..2 {
            pa.deck.add(viper.clone());
        }
        pa.deck.shuffle();
        pa
    }
    pub fn draw_hand(&mut self) {
        let mut id_index = 0 as u8;
        for i in 0..5 {
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
            match self.hand_id.remove(&id) {
                Some((card, card_status)) => self.discard.add(card),
                None => panic!("PlayerArea::discard_hand: id {} is not in hand_id somehow", id)
            }
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
    fn new (all_cards: &HashMap<&str, Card>, deck_cards: Vec<Card>) -> GameState {
        let scout = all_cards.get("scout").expect("cards need a scout!");
        let viper = all_cards.get("viper").expect("cards need a viper!");
        let mut gs = GameState {
            player1: PlayerArea::new(scout, viper),
            player2: PlayerArea::new(scout, viper),
            current_player: Player1,
            trade_row: CardStack::empty(),
            explorers: 10,
            scrapped: CardStack::empty(),
            trade_row_stack: {
                let mut stack = CardStack::new(deck_cards);
                stack.shuffle();
                stack
            },
        };
        gs.fill_trade_row();
        gs
    }
    fn fill_trade_row(&mut self) {
        // todo: number of cards in trade row hard-coded
        let left = 5 - self.trade_row.len();
        for i in 0..left {
            match self.trade_row_stack.draw() {
                None => break,
                Some(card) => self.trade_row.add(card)
            }
        }
    }
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

pub type ConditionFunc = Box<dyn FnMut(&GameState, u8) -> bool>;

pub fn validate_condition(name: &str) -> Option<String> {
    match get_condition(name) {
        Some(_) => None,
        None => format!("Invalid condition: {}", name)
    }
}

pub fn validate_action(name: &str) -> Option<String> {
    match get_action(name) {
        Some(_) => None,
        None => format!("Invalid action: {}", name)
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
                        game.player1.discard.add(Card {
                            cost: 255,
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