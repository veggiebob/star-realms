use std::ops::{AddAssign, Add};
use crate::game::{Goods, GameState, RelativePlayer, HandId};
use crate::game::util::Failure::{Succeed, Fail};
use crate::game::util::Failure;
use crate::game::RelativePlayer::Opponent;
use crate::game::components::card::{Base, Card};
use std::collections::HashSet;
use crate::parse::parse_goods;
use std::fmt::{Display, Formatter};
use ansi_term::Color;
use crate::game::components::faction::Faction;
use std::rc::Rc;

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
    pub describe: Box<dyn Fn(u32) -> String>,
    /// enum that shows how to get the config value (u32)
    pub config_method: ActionConfigMethod
}

pub trait ConfigSupplier {
    /// get a config value (u32) for an action based on this Config object
    fn get_config(&self, game: &GameState, config: &Config) -> u32;
}

//todo: are there any instances where a Range or Set would be used, and need to specify which
// player picks the config? If so, there should be a "by" player abstracted into Config as
// a sibling to ActionConfigMethod
pub enum ActionConfigMethod {
    /// low: u32, high: u32
    /// config should be a number in the range [low..high] inclusive
    Range(u32, u32),

    /// set: contains all the id's that are possible: one is chosen
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

    /// num: u32, by: u32
    /// num = number of trade row cards to pick
    /// by = player that is picking them
    PickTradeRowCards(u32, RelativePlayer)
}

/// FnMut(game, hand_id /* of card */) -> bool
pub type ConditionFunc = Box<dyn FnMut(&GameState, &u32) -> bool>;

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

/// determines if an condition key string signals the "scrap" condition
/// (appears as a trash can on the actual cards)
pub fn is_trash_cond(cond: &String) -> bool {
    if let "trash" | "scrap" = cond.as_str() {
        true
    } else {
        false
    }
}
pub fn is_free_cond(cond: &String) -> bool {
    match cond.as_str() {
        "any" | "free" => true,
        _ => false
    }
}
pub fn get_condition(name: String) -> Option<ConditionFunc> {
    match name.as_str() {
        _ if is_free_cond(&name) => Some(Box::new(|_, _| true)),
        _ if is_trash_cond(&name) => Some(Box::new(
            |game, id| {
                game.get_current_player().hand_id.get(id)
                    .expect("trash condition: bad id supplied")
                    .1.scrapped
            }
        )),
        // example: "syn t" for synergy with Trade Federation
        _ if name.starts_with("syn") => Some(Box::new({
                let n = name.clone();
                move |game, id| match &(n.as_str()[n.len()-1..].parse()) {
                    Ok(p) => game.get_current_player()
                        .get_card_in_hand(id)
                        .expect("synergy condition: bad id supplied")
                        .0.synergizes_with.contains(p),
                    Err(e) => panic!("'{}' is not a valid condition! {}", &n, e)
                }
            })
        ),
        _ => None
    }
}

pub fn get_action(name: &String) -> Option<(ActionMeta, ActionFunc)> {
    // signal to be a good
    if name.starts_with("G") {
        if let Some(goods) = parse_goods(name.as_str()) {
            let action = get_good_action(goods);
            return Some(
                (
                    ActionMeta {
                        description: "gives some amount of trade, authority, and combat".to_string(),
                        config: None
                    },
                    action
                )
            )
        }
    }
    let pattern = regex::Regex::new(r"draw(\s\d)?").unwrap();
    if pattern.is_match(name.as_str()) {
        if let Some(captures) = pattern.captures(name) {
            if let Some(n) = captures.get(1) {
                if let Ok(n) = n.as_str().parse::<u32>() {
                    return Some(
                        (
                            ActionMeta {
                                description: format!("Draw {} cards from your deck", &n),
                                config: None
                            },
                            Box::new(move |game, _| {
                                for _ in 0..n {
                                    game.get_current_player_mut().draw_into_hand();
                                }
                                Succeed
                            })
                        )
                    )
                }
            } else {
                return Some(
                    (
                        ActionMeta {
                            description: "Draw a card from your deck".to_string(),
                            config: None
                        },
                        Box::new(|game, _| {
                            game.get_current_player_mut().draw_into_hand();
                            Succeed
                        })
                    )
                )
            }
        }
    }
    let pattern = regex::Regex::new(r"scrap trade row( \d)?").unwrap();
    if pattern.is_match(name.as_str()) {
        if let Some(captures) = pattern.captures(name) {
            match captures.get(1) {
                Some(n) => {
                    let n = n.as_str().parse::<u32>().unwrap();
                    let n_copy = n.clone();
                    return Some(
                        (
                            ActionMeta {
                                description: format!(
                                    "Scrap up to {} cards in the trade row",
                                    n.clone()
                                ),
                                config: Some(Config {
                                    describe: Box::new(move |_: u32| format!(
                                        "Choosing {} cards to scrap in the trade row",
                                        n_copy
                                    )),
                                    config_method: ActionConfigMethod::PickTradeRowCards(
                                        n.clone(),
                                        RelativePlayer::Current
                                    )
                                }),
                            },
                            Box::new(|game, cfg| {
                                let cards = game.unpack_multi_trade_row_card_selection(&cfg);
                                let cards = game.remove_cards_from_trade_row(cards);
                                for c in cards {
                                    game.scrapped.add(c);
                                }
                                // todo: AAAA MAGIC NUMBERS
                                game.fill_trade_row(5);
                                Succeed
                            })
                            )
                    )
                },
                None => {
                    return Some(
                        (
                            ActionMeta {
                                description: "Scrap a card in the trade row".to_string(),
                                config: Some(Config {
                                    describe: Box::new(|_| "Pick a card from the trade row".to_string()),
                                    config_method: ActionConfigMethod::PickTradeRowCards(1, RelativePlayer::Current)
                                })
                            },
                            Box::new(|game, cfg| {
                                // same implementation even though it's just one card, idc
                                let cards = game.unpack_multi_trade_row_card_selection(&cfg);
                                let cards = game.remove_cards_from_trade_row(cards);
                                for c in cards {
                                    game.scrapped.add(c);
                                }
                                // todo: AAAA MAGIC NUMBERS
                                game.fill_trade_row(5);
                                Succeed
                            })
                        )
                    )
                }
            }
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
                        describe: Box::new(|_| "hand id of card to be discarded".to_string()),
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
                        describe: Box::new(|_| "hand id of the base to be destroyed".to_string()),
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
                                    Failure::Fail(msg) => Failure::Fail(
                                        format!("Unable to discard this card because: {}", msg))
                                }
                            }
                        }
                    }
                })
            )
        ),
        "stealth needle" => Some(
            (
                ActionMeta {
                    description: "Copy any other ship in your hand".to_string(),
                    config: Some(Config {
                        describe: Box::new(|_| "The card to copy".to_string()),
                        config_method: ActionConfigMethod::PickHandCard(
                            RelativePlayer::Current,
                            RelativePlayer::Current
                        )
                    })
                },
                Box::new(|game, cfg: HandId| {
                    // turns out this is not actually a problem if you select another stealth
                    // needle or itself
                    // because even though you can theoretically get an infinite amount of
                    // stealth needles, you cannot actually
                    // increase the number of non-stealth needle
                    // cards that you can copy
                    // so it's not really a loophole
                    // unless you crash the game from a memory overflow?
                    let card = match game
                        .get_current_player()
                        .get_card_in_hand(&cfg) {
                        Some((c, _)) => c,
                        None => return Fail("Not a valid id".to_string())
                    };
                    let mut card = card.clone();
                    card.synergizes_with.insert(Faction::Mech);
                    let id = game.get_current_player_mut().give_card_to_hand(card);
                    game.get_current_player_mut().plan_scrap(&id).unwrap();
                    Succeed
                })
            )
        ),
        "acquire no cost" => Some(
            (
                ActionMeta {
                    description: "Acquire any ship without paying \
                        its cost and put it on top of your deck".to_string(),
                    config: Some(Config {
                        describe: Box::new(|_| "The ship to acquire".to_string()),
                        config_method: ActionConfigMethod::Range(0, 4)
                    })
                },
                Box::new(|game, cfg| {
                    let cl = Rc::clone(&game.card_library);
                    match game.trade_row.remove(cfg as usize) {
                        Some(id) => {
                            let card = cl.as_card(&id);
                            if let None = card.base {
                                game.get_current_player_mut().deck.add((*card).clone());
                                Succeed
                            } else {
                                // make sure to add it back
                                game.trade_row.add(id);
                                Fail("Cannot be a base".to_string())
                            }
                        },
                        None => Fail("Not a valid id".to_string())
                    }
                })
            )
        ),
        "merc cruiser" => Some(
            (
                ActionMeta {
                    description: "Choose a faction as you play Merc Cruiser.\
                     Merc Cruiser has that faction.".to_string(),
                    config: Some(Config {
                        describe: Box::new(|i| match i {
                            0 => "Mech faction",
                            1 => "Fed faction",
                            2 => "Blob faction",
                            3 | _ => "Star faction",
                        }.to_string()),
                        config_method: ActionConfigMethod::Range(0, 3)
                    })
                },
                Box::new(|game, cfg| {
                    let faction = match cfg {
                        0 => Faction::Mech,
                        1 => Faction::Fed,
                        2 => Faction::Blob,
                        3 | _ => Faction::Star
                    };
                    let card = Card {
                        cost: 0,
                        name: "synergy card".to_string(),

                        base: None,
                        synergizes_with: {
                            let mut tmp = HashSet::new();
                            tmp.insert(faction);
                            tmp
                        },
                        effects: HashSet::new()
                    };
                    let id = game.get_current_player_mut().give_card_to_hand(card);
                    game.get_current_player_mut().plan_scrap(&id).unwrap();
                    Succeed
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

impl Display for Goods {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}> <{}> <{}>",
            Color::Yellow.paint(self.trade.to_string()),
            Color::Blue.paint(self.authority.to_string()),
            Color::Red.paint(self.combat.to_string()))
    }
}
