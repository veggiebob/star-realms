use std::ops::{AddAssign, Add};
use crate::game::{Goods, GameState, RelativePlayer};
use crate::game::util::Failure::Succeed;
use crate::game::util::Failure;
use crate::game::RelativePlayer::Opponent;
use crate::game::components::card::{Base, Card};
use std::collections::HashSet;
use crate::parse::parse_goods;
use crate::game::components::faction::Faction;

/// Effects!

pub type ConfigError = String;

/// FnMut(game, config_value) -> Failure<String>
pub type ActionFunc = Box<dyn FnMut(&mut GameState, u32) -> Failure<ConfigError>>;

pub struct ActionMeta {
    /// description of the action, (probably?) user-friendly
    pub description: String,
    pub config: Option<Config>
}

pub struct Config {
    /// dev-friendly description for each of the config values
    description: Box<dyn Fn(u32) -> String>,
    /// enum that shows how to get the config value (u32)
    config_method: ActionConfigMethod
}

pub trait ConfigSupplier {
    fn get_config(&self, game: &GameState, config: &Config) -> u32;
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

/// FnMut(game, hand_id /* of card */) -> bool
pub type ConditionFunc = Box<dyn FnMut(&GameState, u32) -> bool>;

pub fn validate_condition(name: &String) -> Option<String> {
    match get_condition(name.clone()) {
        Some(_) => None,
        None => Some(format!("Invalid condition: {}", name))
    }
}

pub fn validate_action(name: &String) -> Option<String> {
    match get_action(name) {
        Some(_) => None,
        None => Some(format!("Invalid action: {}", name))
    }
}

pub fn validate_effect((cond, act): (&String, &String)) -> Option<String> {
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
        if let Some(e) = validate_effect((l, r)) {
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

pub fn get_condition(name: String) -> Option<ConditionFunc> {
    match name.as_str() {
        "any" | "free" => Some(Box::new(|_, _| true)),
        "trash" | "scrap" => Some(Box::new(
            |game, id| {
                game.get_current_player().hand_id.get(&id)
                    .expect("trash condition: bad id supplied")
                    .1.scrapped
            }
        )),
        _ if name.starts_with("syn") => Some(Box::new({
                let n = name.clone();
                move |game, id| match &(n.as_str()[n.len()-1..].parse()) {
                    Ok(p) => game.get_current_player()
                        .get_card_in_hand(&id)
                        .expect("synergy condition: bad id supplied")
                        .0.synergizes_with.contains(p),
                    Err(e) => panic!(format!("'{}' is not a valid condition! {}", &n, e))
                }
            })
        ),
        _ => None
    }
}

pub fn get_action(name: &String) -> Option<(ActionMeta, ActionFunc)> {
    // signal to be a good
    if name.starts_with("G") {
        return if let Some(goods) = parse_goods(name.as_str()) {
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
    match name.as_str() {
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
                    Succeed
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
                        None => Failure::Fail(format!("No card with id {}", &cfg)),
                        Some((_, card_status)) => if card_status.in_play {
                            Failure::Fail(
                                format!("Card is in play, player must discard from hand \
                                that has not been revealed"))
                        } else if let Failure::Fail(msg) = opponent.discard_by_id(&cfg) {
                            Failure::Fail(format!("unable to discard hand id {} in opponents hand: {}", &cfg, msg))
                        } else {
                            Succeed
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
                        None => Failure::Fail(format!("No card with id {}", &cfg)),
                        Some((card, card_status)) => {
                            if !&card_status.in_play {
                                Failure::Fail(format!("Card {} must be in play!", &card.name))
                            } else if let None = &card.base {
                                Failure::Fail(format!("Card {} is not a base!", &card.name))
                            } else {
                                match opponent.discard_by_id(&cfg) {
                                    Succeed => Succeed,
                                    Failure::Fail(msg) => Failure::Fail(format!("Unable to discard this card because: {}", msg))
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
        game.get_current_player_mut().goods.authority += goods.authority;
        game.get_current_player_mut().goods.trade += goods.trade;
        game.get_current_opponent_mut().goods.authority -= goods.combat;
        Succeed
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