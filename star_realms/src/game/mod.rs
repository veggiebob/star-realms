extern crate rand;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use components::stack::Stack;

use crate::game::card_library::CardLibrary;
use crate::game::components::{Authority, Coin, Combat};
use crate::game::components::card::{Card, CardStatus};
use crate::game::util::Failure;

use crate::game::effects::{ConfigSupplier, get_condition, get_action, Config, ActionConfigMethod, is_trash_cond};
use crate::game::util::Failure::{Succeed, Fail};

pub mod components;
pub mod card_library;
pub mod effects;
mod util;

type CardStack = Stack<Card>;
pub type HandId = u32;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Goods {
    pub(crate) trade: Coin,
    pub(crate) authority: Authority,
    pub(crate) combat: Combat,
}

#[derive(Debug)]
pub struct PlayerArea {
    discard: CardStack,
    deck: CardStack,
    hand_id: HashMap<HandId, (Card, CardStatus)>, // all cards in hand or in play (including bases)
    turn_data: TurnData,
    scrapped: CardStack,
    goods: Goods,
}

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

    fn select_effect(&self, game: &GameState) -> UserActionIntent<(u32, (String, String))>;

    /// return 0 to attempt to buy an explorer
    fn select_trade_row_card(&self, game: &GameState) -> UserActionIntent<u32>;

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
                authority: 50,
                trade: 0
            },
            turn_data: TurnData::new()
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

    pub fn get_all_hand_card_ids(&self) -> HashSet<u32> {
        let mut set = HashSet::new();
        for (k, _) in self.hand_id.iter() {
            set.insert(*k);
        }
        set
    }

    /// panic if there are more bits than cards in `hand_ids`
    pub fn unpack_multi_card_id(&self, bit_flagged: u32) -> HashSet<u32> {
        let mut ids = HashSet::new();
        let num_cards = f32::log2(bit_flagged as f32).ceil() as u32;
        // println!("unpacking: {}", bit_flagged);
        // println!("there are {} cards", num_cards);
        let hand_ids = self.get_all_hand_card_ids();
        let sorted_ids = {
            let mut tmp: Vec<_> = hand_ids.iter().collect();
            tmp.sort();
            tmp
        };
        if num_cards as usize > sorted_ids.len() {
            panic!("Bit flag contained more bits than there are cards in the hand!");
        }
        for i in 0..num_cards {
            if ((1<<i) & bit_flagged) > 0 {
                ids.insert(*sorted_ids[i as usize]);
            }
        }
        ids
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
            if let None = card {
                break;
            }
            let card = card.unwrap();
            while self.hand_id.contains_key(&id_index) {
                id_index += 1;
            }
            if card.base.is_some() {
                self.plan_discard(&id_index).unwrap();
            }
            self.hand_id.insert(id_index, (card, CardStatus::new()));
        }
    }
    fn get_unused_hand_id(&self) -> HandId {
        let mut id_index = 0;
        while self.hand_id.contains_key(&id_index) {
            id_index += 1;
        }
        id_index
    }

    /// Note: this may not ever be used because each card is handled differently
    /// at the end of a turn. It might be scrapped, discarded, or kept (bases)
    fn discard_hand(&mut self) {
        let keys_vec = {
            let mut tmp = vec![];
            let keys = self.hand_id.keys();
            for id in keys {
                tmp.push(*id);
            }
            tmp
        };
        for id in keys_vec {
            if let Failure::Fail(err) = self.discard_by_id(&id) {
                panic!("PlayerArea::discard_hand: {}", err);
            }
        }
    }
    pub fn end_turn(&mut self) {
        let to_be_scrapped = self.turn_data.to_be_scrapped.clone();
        for id in to_be_scrapped {
            if let Failure::Fail(err) = self.scrap_by_id(&id) {
                panic!("PlayerArea::end_turn: {}", err);
            }
        }
        let to_be_discarded = self.turn_data.to_be_discarded.clone();
        for id in to_be_discarded {
            if let Failure::Fail(err) = self.discard_by_id(&id) {
                panic!("PlayerArea::end_turn: {}", err);
            }
        }
        self.goods.trade = 0;
        self.goods.combat = 0; // todo: aggregate combat then deal it at the end of the turn
        self.turn_data.reset();
    }

    /// Discard this card at the end of the turn.
    /// Cards are planned to be discarded if they are drawn.
    /// Err => not a valid id
    /// (it's ok to plan_discard the same card more than once)
    pub fn plan_discard(&mut self, id: &HandId) -> Result<(), ()> {
        if self.hand_id.contains_key(id) {
            self.turn_data.to_be_discarded.insert(*id);
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn plan_scrap(&mut self, id: &HandId) -> Result<(), ()> {
        if self.hand_id.contains_key(id) {
            self.turn_data.to_be_scrapped.insert(*id);
            Ok(())
        } else {
            Err(())
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
    pub fn scrap_by_id(&mut self, id: &u32) -> Failure<String> {
        match self.hand_id.remove(id) {
            Some((card, _)) => {
                self.scrapped.add(card);
                Failure::Succeed
            },
            None => Failure::Fail(format!("cannot scrap card by id {}!", id))
        }
    }
    fn draw(&mut self) -> Option<Card> {
        if let Some(c) = self.deck.draw() {
            Some(c)
        } else {
            self.discard.shuffle();
            while let Some(c) = self.discard.draw() {
                self.deck.add(c);
            }
            if self.deck.len() > 0 {
                // this *should* never recurse infinitely
                self.draw()
            } else {
                None
            }
        }
    }
    pub fn give_card_to_hand (&mut self, card: Card) {
        self.hand_id.insert(self.get_unused_hand_id(), (card, CardStatus::new()));
    }
}

impl GameState {
    /// panics if there is no scout or viper
    /// this is helpful https://www.starrealms.com/sets-and-expansions/
    pub fn new (card_library: Rc<CardLibrary>) -> GameState {
        let scout = card_library.get_scout().expect("card library needs a scout!");
        let viper = card_library.get_viper().expect("card library needs a viper!");
        let mut gs = GameState {
            player1: PlayerArea::new((*scout).clone(), (*viper).clone(), true),
            player2: PlayerArea::new((*scout).clone(), (*viper).clone(), false),
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


    /// A Result::Err(s) indicates an internal error: STRICTLY UNRECOVERABLE
    /// A Result::Ok(s) indicates a message that should be logged, but not shown to the user
    ///     this case is RECOVERABLE (the function can be run again)
    pub fn advance<T>(&mut self, client: &T) -> Result<String, String>
        where T: ConfigSupplier + UserActionSupplier {
        println!("current player: {:?}", self.current_player);
        println!("{:?}", self.get_current_player().goods);

        let next = client.choose_abstract_action(self);
        match next {
            AbstractPlayerAction::CardEffects =>
                if let UserActionIntent::Continue((card_id, (cond_s, act_s)))
            // select to either exit, or continue with an effect
                = client.select_effect(self) {
                // then parse the condition and action of this effect
                let mut cond = get_condition(cond_s.clone())
                    .expect(
                        format!(
                            "GameState.advance(): bad selection condition {}. \
                        It might be a good idea to validate cards before hand.", &cond_s)
                            .as_str());
                let (action_meta, mut action_func) = get_action(&act_s)
                    .expect(
                        format!("GameState.advance(): bad selection action {}. \
                        It might be a good idea to validate cards before hand.", &act_s)
                            .as_str());
                // evaluate the condition
                if cond(self, &card_id) {
                    // if true, run the action
                    // println!("cond succeeded! running action...");
                    match action_func(self,
                                      match action_meta.config {
                                          Some(config) => client.get_config(self, &config),
                                          _ => 0,
                                      }) {
                        // if the action fails, then a bad config was passed in.
                        // perhaps we can report these better
                        Fail(msg) => Err(
                            format!(
                                "Unable to complete action to {}. {}",
                                action_meta.description.clone(),
                                msg)
                        ),
                        // if it succeeds, make sure to consume the effect
                        Succeed => {
                            match self.get_current_player_mut().get_card_in_hand_mut(&card_id) {
                                Some((_, card_status)) => {
                                    card_status.use_effect(&(cond_s, act_s));
                                    Ok("Effect was used and consumed".to_string())
                                }
                                None => Err(
                                    format!(
                                        "Card id {} is not one of {:?}",
                                        card_id,
                                        self.get_current_player().hand_id.keys()))
                            }
                        }
                    }
                } else {
                    // if the condition is not true, report the mistake to the client, or user?
                    let s = "This effect is not possible at the moment.".to_string();
                    client.on_feedback(Feedback::Invalid(s.clone()));
                    Ok(s)
                }
            } else {
                Ok("Canceled card effect selection".to_string())
            }
            AbstractPlayerAction::TradeRow =>
                if let UserActionIntent::Continue(index) = client.select_trade_row_card(self) {
                    if index == 0 {
                        return if self.explorers == 0 {
                            let s = "There are no explorers left".to_string();
                            client.on_feedback(Feedback::Invalid(s.clone()));
                            Ok(s)
                        } else {
                            let explorer = (*self.card_library.get_explorer().unwrap()).clone();
                            if explorer.cost > self.get_current_player().goods.trade {
                                client.on_feedback(
                                    Feedback::Invalid(
                                        "Not enough trade to buy an explorer".to_string()));
                                Ok("Cannot buy explorer".to_string())
                            } else {
                                self.explorers -= 1;
                                self.get_current_player_mut().goods.trade -= explorer.cost;
                                self.get_current_player_mut().discard.add(explorer);
                                Ok("Bought an explorer".to_string())
                            }
                        }
                    }
                    let index = index - 1; // the 0th place was just for explorers, a special case
                    let card_id = self.trade_row.peek(index as usize);
                    if let Some(card_id) = card_id {
                        let card = self.card_library.as_card(card_id);
                        if card.cost <= self.get_current_player().goods.trade {
                            self.trade_row.remove(index as usize).unwrap();
                            let success_message = format!("{:?} acquired {}", self.current_player, &card.name);
                            let player = self.get_current_player_mut();
                            player.goods.trade -= card.cost;
                            player.discard.add((*card).clone());
                            Ok(success_message)
                        } else {
                            let s = format!("Cannot purchase card {} since the cost is more \
                                trade than the current player owns. {} > {}", card.name, card.cost,
                                            self.get_current_player().goods.trade);
                            client.on_feedback(Feedback::Invalid(s.clone()));
                            Ok(s)
                        }
                    } else {
                        Err(format!("Client error: index was out of bounds. Cannot peek card \
                            at {} in a trade row of length {}", index, self.trade_row.len()))
                    }
                } else {
                    Ok("Canceled trade row purchase".to_string())
                }
            AbstractPlayerAction::EndTurn => {
                // the client chooses to exit, and hand over the turn.
                // todo: warn them if they haven't completed all their effects with Feedback::Info
                //     and client.on_feedback()
                // todo: automatically exit turn if all effects have been completed
                self.get_current_player_mut().end_turn();
                self.get_current_player_mut().draw_hand(5);
                self.flip_turn();
                Ok("Turn was ended".to_string())
            }
            AbstractPlayerAction::TrashCard => {
                let card_id = client.get_config(self, &Config {
                    describe: Box::new(|_| "The card to be scrapped".to_string()),
                    config_method: ActionConfigMethod::PickHandCard(
                        RelativePlayer::Current,
                        RelativePlayer::Current
                    )
                });
                let (card, card_status) = self.get_current_player_mut()
                    .get_card_in_hand_mut(&card_id)
                    .unwrap_or_else(|| panic!("Client: supplied bad card id {}", &card_id));
                if card.effects.iter().any(|(c, _)| is_trash_cond(c)) {
                    card_status.scrapped = true;
                    client.on_feedback(
                        Feedback::Info(
                            "This card's trash effect can now be used.".to_string()));
                    Ok("Scrapped a card using the trash action".to_string())
                } else {
                    let s = "This card cannot be scrapped".to_string();
                    client.on_feedback(Feedback::Invalid(s.clone()));
                    Ok(s)
                }
            }
        }
    }
}