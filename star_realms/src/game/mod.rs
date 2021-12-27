extern crate rand;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use components::card::active_card::IdCardCollection;
use components::stack::SimpleStack;

use crate::game::actions::client_comms::{Client, ClientActionOptionQuery, ClientQuery, TextStyle, StyledText, ClientActionOptionResponse};
use crate::game::card_library::CardLibrary;
use crate::game::components::{Authority, Coin, Combat, Goods};
use crate::game::components::card::{Card, CardRef};
use crate::game::components::card::active_card::ActiveCard;
use crate::game::components::card::details::{CardSource, Action, PlaySet, Play, Actionable};
use crate::game::RelativePlayer::{Current, Opponent};
use crate::game::util::{Failure, Join};
use crate::game::util::Failure::{Fail, Succeed};
use crate::game::components::stack::{Stack, move_all_to};
use crate::game::components::card::details::Base::Outpost;
use crate::game::components::card::in_game::{ActivePlay, Trigger};
use ansi_term::Color;
use std::borrow::BorrowMut;

pub mod components;
pub mod card_library;
pub mod util;
pub mod actions;
pub mod requirements;

type CardStack = SimpleStack<CardRef>;

#[derive(Debug)]
pub struct TurnData {
    total_combat: Combat,
    money: Coin,
    triggers: Vec<Trigger>
}

impl TurnData {
    pub fn new() -> TurnData {
        TurnData {
            total_combat: 0,
            money: 0,
            triggers: vec![]
        }
    }
    pub fn reset(&mut self)  {
        self.total_combat = 0;
        self.triggers = vec![];
        self.money = 0;
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
    authority: Authority
}

impl PlayerArea {
    pub fn new(scout: CardRef, viper: CardRef, starting_health: Authority) -> PlayerArea {
        let mut pa = PlayerArea {
            hand: IdCardCollection::new(SimpleStack::empty()),
            table: IdCardCollection::new(SimpleStack::empty()),
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
        ActiveCard {
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

    pub fn deal_damage(&mut self, damage: Authority) -> bool {
        if damage >= self.authority {
            self.authority = 0;
            true
        } else {
            self.authority -= damage;
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
        let mut ids: Vec<_> = ids.iter().filter(|&&i| (i as usize) < self.trade_row.len()).collect();
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

    pub fn get_stack(&self, card_source: CardSource) -> &dyn Stack<CardRef> {
        match card_source {
            CardSource::Deck(player) => match player {
                Current => &self.get_current_player().deck,
                Opponent => &self.get_current_opponent().deck
            },
            CardSource::Discard(player) => match player {
                Current => &self.get_current_player().discard,
                Opponent => &self.get_current_opponent().discard
            },
            CardSource::Hand(player) => match player {
                Current => &self.get_current_player().hand,
                Opponent => &self.get_current_opponent().hand
            },
            CardSource::Table(player) => match player {
                Current => &self.get_current_player().table,
                Opponent => &self.get_current_opponent().table
            },
            CardSource::TradeRow => &self.trade_row
        }
    }

    pub fn get_mut_stack(&mut self, card_source: CardSource) -> &mut dyn Stack<CardRef> {
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
            CardSource::Table(player) => match player {
                Current => &mut self.get_current_player_mut().table,
                Opponent => &mut self.get_current_opponent_mut().table
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

    pub fn advance<T: Client>(&mut self, client: &mut T) {
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
            let cards = &self.get_current_player().hand; // todo: also gather base plays somehow
            let mut plays = hashmap!{};
            let mut idx: u32 = 0;
            for card_data in <IdCardCollection as Stack<ActiveCard>>::iter(&cards) {
                let card: &ActiveCard = card_data; // for the sake of autocomplete
                if let Some(playset) = &card.card.content {
                    if !plays.contains_key(&idx) {
                        plays.insert(idx, vec![]);
                    }
                    for play in playset.iter() {
                        plays.get_mut(&idx).unwrap().push(ActivePlay::new(play));
                    }
                }
                idx += 1;
            }

            let plays = {
                let mut tmp: Vec<_> = (0..idx).into_iter().map(|_| vec![]).collect();
                for idx in 0..idx {
                    let v = plays.get(&idx).unwrap();
                    v.into_iter()
                        .map(|p| p.play.actn.name.clone())
                        .for_each(|n|
                            tmp.get_mut(idx as usize).unwrap().push(n)
                        );
                }
                tmp
            };

            let response = client.resolve_action_query(ClientQuery {
                action_query: ClientActionOptionQuery::PlaySelection(plays),
                performer: self.current_player
            }, &self);

            if let ClientActionOptionResponse::PlaySelection(play) = response {
                // if we get a valid response type,
                if let Some((card_idx, play_idx)) = play {
                    // and a selection was made,
                    if let Some(play) = <IdCardCollection as Stack<CardRef>>::get(&self.get_current_player().hand, card_idx as usize)
                        .and_then(|card: &CardRef| card.content.as_ref())
                        .and_then(|playset: &PlaySet| playset.get(play_idx as usize))
                        .and_then(|play: &Play| Some(play.clone())) {
                        // and the selection actually produces output,
                        // then we can continue to work with the play.
                        let mut play: Play = play; // IDE issues :')

                        // evaluate the condition
                        let cond_ok = if play.cond.is_some() {
                            todo!()
                        } else {
                            true
                        };

                        if cond_ok {
                            GameState::broadcast_message(client, format!("Playing the action '{}'", &play.actn.name).into());
                            self.handle_action(client, &mut play.actn.item);
                        }
                    }
                } else {
                    // no selection was made (no cards / plays available)
                    GameState::message_player(
                        client,
                        &self.current_player,
                        "No plays left. Continuing, I guess!");
                }
            } else {
                // wrong response type returned
                GameState::broadcast_message(client,
                    StyledText {
                        style: TextStyle::error(),
                        text: "Client Error: Not a valid response type. Expected PlaySelection(play)".to_string()
                    }
                );
            }

        }

        // 2. deal damage to the opponent
        {
            let current_combat = self.get_current_player().turn_data.total_combat;
            let current_player = self.current_player;
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
                    GameState::broadcast_message(
                        client,
                        format!("work in progress! you should pick the bases to do damage to!").into()
                    );
                    // todo: choose which things to destroy
                }
            } else {
                GameState::broadcast_message(
                    client,
                    format!(
                        "{:?} deals {} damage to {:?}",
                        current_player,
                        current_combat,
                        current_player
                    ).into()
                );
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
                    if res.is_some() {
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

    /// performs, based on the many parameters and choices involved
    fn handle_action<C: Client>(&mut self, client: &mut C, action: &mut Action) {
        match action {
            Action::Sequential(action_1, action_2) => {
                // this performs two actions :)
                // with join :(
            },
            Action::Unit(action) => {
                match action {
                    Join::Unit(ref mut action) => {
                        let client_info = self.query_actionable_information(client, &action);
                        let mut run_func = (*action.run).borrow_mut();
                        let run = (**run_func)(self, client_info);
                        // todo: finish
                    },
                    Join::Union(actions) => {
                        for action in actions {
                            let mut action = Action::Unit(*action.clone()); // collapsed!
                            self.handle_action(client, &mut action);
                        }
                    },
                    Join::Disjoint(actions) => {
                        // ask the user to choose one of these options
                        let i = GameState::anon_choice(client, actions.len(), &self.current_player);
                        let mut action = actions.get(i).unwrap();
                        let mut action = Action::Unit(*action.clone());  // collapsed!
                        self.handle_action(client, &mut action);
                    }
                }
            }
        }
    }

    /// handle action in its unit form: actionable
    fn query_actionable_information<C: Client>(&mut self, client: &mut C, actionable: &Actionable) -> Option<ClientActionOptionResponse> {
        if let Some(query) = &actionable.client_query {
            let query = self.resolve_query_choice(client, query);
            let query = ClientQuery {
                action_query: query.clone(),
                performer: self.current_player,
            };
            Some(client.resolve_action_query(query, self))
        } else {
            None
        }
    }

    fn resolve_query_choice<'a, C: Client>(&self, client: &C, q: &'a Join<ClientActionOptionQuery>) -> &'a ClientActionOptionQuery {
        match q {
            Join::Unit(query) => {
                query
            },
            Join::Union(_) => {
                panic!("GameState::handle_actionable: Join::Union doesn't make sense in this scenario");
            },
            Join::Disjoint(queries) => {
                let choice = GameState::anon_choice(client, queries.len(), &self.current_player);
                let query = queries.get(choice).unwrap();
                self.resolve_query_choice(client, query)
            }
        }
    }

    /// Ask the client to choose a number.
    /// ideally this shouldn't ever be used,
    /// because the user has no idea what they're choosing.
    /// but it's an abstraction I need right now to implement things
    /// as fast as possible
    fn anon_choice<C: Client>(client: &C, num: usize, player: &Player) -> usize {
        let choice_range = (0..num).map(|x| (x.clone().to_string(), x)).collect::<Vec<_>>();
        let options: Vec<_> = choice_range.iter().map(|(s, n)| (s.as_str(), n)).collect();
        let options = hashmap!{
                    *player => Some(options)
                };
        let response: Option<&usize> = client.alert(
            &hashmap!{
                        *player => "Choose one option"
                    },
            &options,
                TextStyle::plain()
        );
        *response.unwrap()
    }

    fn handle_query<C: Client>(client: &C, query: &ClientActionOptionQuery) -> ClientActionOptionResponse {
        todo!()
    }

    fn all_players<T: Clone>(item: T) -> HashMap<Player, T> {
        hashmap!{
            Player::Player1 => item.clone(),
            Player::Player2 => item
        }
    }

    fn message_player<C: Client, T: Into<StyledText>>(client: &C, player: &Player, message: T) {
        let message = message.into();
        client.alert::<()>(
            &hashmap!{
                *player => message.text.as_str()
            },
            &GameState::all_players(None),
            message.style
        );
    }

    fn broadcast_message<C: Client>(client: &C, message: StyledText) {
        client.alert::<()>(
            &GameState::all_players(message.text.as_str()),
            &GameState::all_players(None),
            message.style);
    }

}