extern crate rand;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use components::card::active_card::IdCardCollection;
use components::stack::SimpleStack;

use crate::game::actions::client_comms::{Client, ClientActionOptionQuery, ClientQuery, TextStyle, StyledText};
use crate::game::card_library::CardLibrary;
use crate::game::components::{Authority, Coin, Combat, Goods};
use crate::game::components::card::{Card, CardRef};
use crate::game::components::card::active_card::ActiveCard;
use crate::game::components::card::details::{CardSource, Action};
use crate::game::RelativePlayer::{Current, Opponent};
use crate::game::util::Failure;
use crate::game::util::Failure::{Fail, Succeed};
use crate::game::components::stack::{Stack, move_all_to};
use crate::game::components::card::details::Base::Outpost;
use crate::game::components::card::in_game::ActivePlay;

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
    played_this_turn: HashSet<HandId>,
    total_combat: Combat,
    money: Coin
}

impl TurnData {
    pub fn new() -> TurnData {
        TurnData {
            to_be_scrapped: HashSet::new(),
            to_be_discarded: HashSet::new(),
            played_this_turn: HashSet::new(),
            total_combat: 0,
            money: 0
        }
    }
    pub fn reset(&mut self)  {
        self.to_be_discarded = HashSet::new();
        self.to_be_scrapped = HashSet::new();
        self.played_this_turn = HashSet::new();
        self.total_combat = 0;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
    ids: HashSet<HandId>,
    authority: Authority
}

impl PlayerArea {
    pub fn new(scout: CardRef, viper: CardRef, starting_health: Authority) -> PlayerArea {
        let mut pa = PlayerArea {
            hand: IdCardCollection::new(SimpleStack::empty(), &HashSet::new()),
            table: IdCardCollection::new(SimpleStack::empty(), &HashSet::new()),
            turn_data: TurnData::new(),
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
            ids: HashSet::new(),
            authority: starting_health
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
                    move_all_to(&mut self.discard, &mut self.deck);
                    self.draw_card()
                },
                x => x
            }
        }
    }

    pub fn draw_card_into_hand(&mut self) -> Failure<String> {
        match self.draw_card() {
            Some(card) => {
                let active_card = self.activate_card(card, true, true);
                self.hand.add(active_card);
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
        self.update_ids();
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

    fn update_ids(&mut self) {
        let mut ids = HashSet::new();
        for id in self.hand.get_ids() {
            ids.insert(id);
        }
        for id in self.table.get_ids() {
            ids.insert(id);
        }
        self.ids = ids;
    }

    pub fn draw_cards_into_hand(&mut self, num: usize) -> Failure<String> {
        for i in 0..num {
            if let Fail(_) = self.draw_card_into_hand() {
                return Fail(format!("Failed to draw {} cards, drew {}.", num, i));
            }
        }
        Succeed
    }

    pub fn deal_damage(&mut self, damage: Authority) -> bool {
        if damage >= self.authority {
            self.authority = 0;
            true
        } else {
            self.authority -= self.authority;
            false
        }
    }
}

impl GameState {
    /// ## Panic
    /// If there is no scout or viper (because CardLibrary can only be created using them)
    /// ## Other
    /// this is helpful https://www.starrealms.com/sets-and-expansions/
    pub fn new (card_library: Rc<CardLibrary>, starting_health: Authority) -> GameState {
        let scout = card_library.get_scout().expect("card library needs a scout!");
        let viper = card_library.get_viper().expect("card library needs a viper!");
        let mut gs = GameState {
            player1: PlayerArea::new(Rc::clone(&scout), Rc::clone(&viper), starting_health),
            player2: PlayerArea::new(Rc::clone(&scout), Rc::clone(&viper), starting_health),
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

    pub fn get_stack_mut(&mut self, card_source: CardSource) -> &mut dyn Stack<CardRef> {
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
                Current => &mut self.get_current_player_mut().hand,
                Opponent => &mut self.get_current_opponent_mut().hand
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

    pub fn advance<T: Client>(&mut self, client: &T) {
        // not sure why I decided to put multiple receivers

        // notes: each part of this function (there are 5) should
        //   be able to be completed entirely within that scope (esp. #1), and it should
        //   only exit prematurely if it encounters a *very* fatal error

        // Turn layout:
        // (should have up to 5 cards in hand)
        // 1. take any of the actions on any of the cards, provided that it is able
        //  - keep a running sum of damage (combat)
        {
            // gather the plays
            let cards = &self.get_current_player().hand;
            let mut plays = vec![];
            for card_data in <IdCardCollection as Stack<ActiveCard>>::iter(&cards) {
                let card: &ActiveCard = card_data; // for the sake of autocomplete
                if let Some(playset) = &card.card.content {
                    for play in playset.iter() {
                        plays.push(ActivePlay::new(play));
                    }
                }
            }


            // print the options to the player
            let mut plays_string = vec![];
            let mut idx = 0;
            for play in plays.iter() {
                plays_string.push(
                    format!("{}. {}\n",
                        idx,
                        play.play.actn.name.clone()
                    )
                );
                idx += 1;
            }

            // this section is unfinished, and needs revision
            let msg = format!("There are {} plays available:\n{}", plays.len(), plays_string.concat());
            client.alert::<()>(
            &hashmap! {
                    self.current_player => &*msg
                },
                &GameState::all_players(None),
                TextStyle::plain()
            );
        }

        // 2. deal damage to the opponent
        {
            let current_combat = self.get_current_player().turn_data.total_combat;
            let opponent = self.get_current_opponent_mut();

            // calculate the outpost with the least defense (if there is one)
            let min_defense = opponent.table.cards.iter().filter_map(
                |ac|
                    if let Some(Outpost(defense)) = ac.card.base {
                        Some(defense)
                    } else {
                        None
                    }
            ).min();
            // if there is an outpost, and using the outpost with the minimum defense
            if let Some(d) = min_defense {
                if self.get_current_player().turn_data.total_combat < d {
                    // if the amount of combat is less than the defense of
                    // the lowest outpost, the player can't do any damage
                    GameState::broadcast_message(
                        client,
                        format!(
                            "No damage could be done. {} < {}",
                            self.get_current_player().turn_data.total_combat,
                            d,
                        ).into()
                    );
                } else {
                    // todo: choose which things to destroy
                }
            } else {
                let dead = opponent.deal_damage(current_combat);
                if dead {
                    // tell both players that the game is over,
                    // and give them the option to quit, or ... quit.
                    let message = format!(
                        "Game is over! {:?} defeated {:?}",
                        self.current_player,
                        self.current_player.reverse(),
                    );
                    let message = message.as_str();
                    let op = true;
                    let options = Some(vec![("Ok", &op)]);

                    let res = client.alert(
                        &GameState::all_players(message),
                        &GameState::all_players(options),
                        TextStyle::attention()
                    );
                    if let Some(x) = res {
                        let x = client.alert::<()>(
                            {
                                let msg = "Quitting the game.";
                                &hashmap! {
                                    self.current_player => msg,
                                    self.current_player.reverse() => msg
                                }
                            },
                            &hashmap! {
                                self.current_player => None,
                                self.current_player.reverse() => None
                            },
                            TextStyle::plain()
                        );
                    }
                }
            }
        }

        // 3. discard all cards in hand
        //  - discard all cards scheduled to be discarded
        //  - scrap all cards scheduled to be scrapped
        // current.hand.move_all_to(&mut current.discard);
        {
            let mut current = self.get_current_player_mut();
            move_all_to(&mut current.hand, &mut current.discard);
        }

        // 4. draw 5 cards into hand
        {
            self.get_current_player_mut().draw_cards_into_hand(5);
        }

        // 5. flip turn
        self.flip_turn();
    }

    fn all_players<T: Clone>(item: T) -> HashMap<Player, T> {
        hashmap!{
            Player::Player1 => item.clone(),
            Player::Player2 => item
        }
    }

    fn broadcast_message<C: Client>(client: &C, message: StyledText) {
        client.alert::<()>(
            &GameState::all_players(message.text.as_str()),
            &GameState::all_players(None),
            message.style);
    }

}