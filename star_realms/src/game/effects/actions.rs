use core::option::Option;
use core::option::Option::{None, Some};
use core::result::Result::Ok;
use std::collections::{HashSet};

use crate::game::{effects, GameState, Goods, HandId, RelativePlayer};
use crate::game::components::card::{Base, Card, Effects};
use crate::game::components::faction::Faction;
use crate::game::RelativePlayer::Opponent;
use crate::game::util::Failure::{Fail, Succeed};
use crate::game::util::Failure;
use crate::game::components::{Authority, Coin, Combat};
use crate::game::effects::{PreConfig, PreConfigMeta, PreConfigType, UserConfigMeta};
use std::rc::Rc;

pub struct ActionCreator {
    pub meta: ActionConfig,
    pub generator: ActionGenerator
}

type ActionGenerator = Box<dyn FnMut(PreConfig) -> Action>;

/// describes how the action can be created
pub struct ActionConfig {
    pub pre_config_meta: Option<PreConfigMeta>
}

pub struct Action {
    pub meta: ActionExecution,
    pub action: ActionFunc
}

pub type ActionFunc = Box<dyn FnMut(&mut GameState, ActionExecutionConfig) -> Failure<String>>;
pub type ActionExecutionConfig = u32;

/// description of properties during execution
pub struct ActionExecution {
    /// description of the action, (probably?) user-friendly
    pub description: String,
    pub config: Option<UserConfigMeta>,
}

pub fn get_action(name: &String) -> Option<ActionCreator> {
    match name.as_str() {
        "scrap trade row card" => Some(ActionCreator {
            meta: ActionConfig { pre_config_meta: None },
            generator: Box::new(|cfg| {
                Action {
                    meta: ActionExecution {
                        description: "Scrap a card in the trade row".to_string(),
                        config: Some(UserConfigMeta {
                            describe: Box::new(|_| "Pick a card from the trade row".to_string()),
                            config_method: ActionConfigMethod::PickTradeRowCards(1, RelativePlayer::Current)
                        })
                    },
                    action: Box::new(|game, cfg| {
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
                }
            })
        }),
        "scrap trade row cards" => Some(ActionCreator {
            meta: ActionConfig {
                pre_config_meta: Some(PreConfigMeta::all_required(vec![
                    ("cards", PreConfigType::Nat)
                ]))
            },
            generator: Box::new(|cfg| {
                let n = &cfg.get_nat("cards");
                Action {
                    meta: ActionExecution {
                        description: format!(
                            "Scrap up to {} cards in the trade row",
                            *n
                        ),
                        config: Some(UserConfigMeta {
                            describe: Box::new(move |_: u32| format!(
                                "Choosing {} cards to scrap in the trade row",
                                *n
                            )),
                            config_method: ActionConfigMethod::PickTradeRowCards(
                                *n,
                                RelativePlayer::Current
                            )
                        }),
                    },
                    action: Box::new(|game, cfg| {
                        let cards = game.unpack_multi_trade_row_card_selection(&cfg);
                        let cards = game.remove_cards_from_trade_row(cards);
                        for c in cards {
                            game.scrapped.add(c);
                        }
                        // todo: AAAAaaaaa MAGIC NUMBERS
                        game.fill_trade_row(5);
                        Succeed
                    })
                }
            })
        }),
        "goods" => Some(
            ActionCreator {
                meta: ActionConfig {
                    pre_config_meta: Some(
                        PreConfigMeta::all_required(
                            vec![
                                ("combat", PreConfigType::Nat),
                                ("trade", PreConfigType::Nat),
                                ("authority", PreConfigType::Nat)
                            ]
                        )
                    )
                },
                generator: Box::new(|cfg: PreConfig| {
                    let authority = cfg.get_nat("authority") as Authority;
                    let trade = cfg.get_nat("trade") as Coin;
                    let combat = cfg.get_nat("combat") as Combat;
                    Action {
                        meta: ActionExecution {
                            description: format!(
                                "Gives {} trade, {} combat, and {} authority",
                                trade, combat, authority),
                            config: None
                        },
                        action: get_good_action(Goods {
                            authority,
                            trade,
                            combat
                        })
                    }
                })
            }
        ),
        "draw" => Some(
            ActionCreator {
                meta: ActionConfig {
                    pre_config_meta: Some(
                        PreConfigMeta::all_required(vec![("cards", PreConfigType::Nat)])
                    )
                },
                generator: Box::new(|cfg| {
                    let cards = cfg.get_nat("cards");
                    Action {
                        meta: ActionExecution {
                            description: format!("Draw {} cards", &cards),
                            config: None
                        },
                        action: Box::new(|game| {
                            game.get_current_player().draw_hand(cards as u8);
                            Succeed
                        })
                    }
                })
            }
        ),
        "discard" => Some(
            ActionCreator {
                meta: ActionConfig {
                    pre_config_meta: Some(PreConfigMeta::all_required(vec![("cards", PreConfigType::Nat)]))
                },
                generator: Box::new(|cfg| {
                    Action {
                        meta: ActionExecution {
                            description: "opponent discards a card".to_string(),
                            config: Some(UserConfigMeta {
                                describe: Box::new(|_| "hand id of card to be discarded".to_string()),
                                config_method: ActionConfigMethod::PickHandCard(Opponent, Opponent)
                            })
                        },
                        action: Box::new(|game: &mut GameState, cfg| {
                            let opponent = game.get_current_opponent_mut();
                            match opponent.hand_id.get(&cfg) {
                                None => Fail(format!("No card with id {}", &cfg)),
                                Some((_, card_status)) => if card_status.in_play {
                                    Fail(
                                        format!("Card is in play, player must discard from hand \
                                        that has not been revealed"))
                                } else if let Fail(msg) = opponent.discard_by_id(&cfg) {
                                    Fail(format!("unable to discard hand id {} in opponents hand: {}", &cfg, msg))
                                } else {
                                    Succeed
                                }
                            }
                        })
                    }
                })
            }
        ),
        "destroy target base" => Some(
            ActionCreator {
                meta: ActionConfig::no_config(),
                generator: Box::new(|cfg| {
                    Action {
                        meta: ActionExecution {
                            description: "destroy any of the opponents bases".to_string(),
                            config: Some(UserConfigMeta {
                                describe: Box::new(|_| "hand id of the base to be destroyed".to_string()),
                                config_method: ActionConfigMethod::PickHandCard(RelativePlayer::Current, RelativePlayer::Opponent)
                            }),
                        },
                        action: Box::new(|game: &mut GameState, cfg| {
                            let opponent = game.get_current_opponent_mut();
                            match opponent.hand_id.get(&cfg) {
                                None => Fail(format!("No card with id {}", &cfg)),
                                Some((card, card_status)) => {
                                    if !&card_status.in_play {
                                        Fail(format!("Card {} must be in play!", &card.name))
                                    } else if let None = &card.base {
                                        Fail(format!("Card {} is not a base!", &card.name))
                                    } else {
                                        match opponent.discard_by_id(&cfg) {
                                            Succeed => Succeed,
                                            Fail(msg) => Fail(
                                                format!("Unable to discard this card because: {}", msg))
                                        }
                                    }
                                }
                            }
                        })
                    }
                })
            }
        ),
        "stealth needle" => Some(
            ActionCreator {
                meta: ActionConfig::no_config(),
                generator: Box::new(|cfg| {
                    Action {
                        meta: ActionExecution {
                            description: "Copy any other ship in your hand".to_string(),
                            config: Some(UserConfigMeta {
                                describe: Box::new(|_| "The card to copy".to_string()),
                                config_method: ActionConfigMethod::PickHandCard(
                                    RelativePlayer::Current,
                                    RelativePlayer::Current
                                )
                            })
                        },
                        action: Box::new(|game, cfg: HandId| {
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
                    }
                })
            }
        ),
        "acquire no cost" => Some(
            ActionCreator {
                meta: ActionConfig::no_config(),
                generator: Box::new(|cfg| {
                    Action {
                        meta: ActionExecution {
                            description: "Acquire any ship without paying \
                        its cost and put it on top of your deck".to_string(),
                            config: Some(UserConfigMeta {
                                describe: Box::new(|_| "The ship to acquire".to_string()),
                                config_method: ActionConfigMethod::Range(0, 4)
                            })
                        },
                        action: Box::new(|game, cfg| {
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
                    }
                })
            }
        ),
        "merc cruiser" => Some(
            ActionCreator {
                meta: ActionConfig::no_config(),
                generator: Box::new(|cfg| {
                    Action {
                        meta: ActionExecution {
                            description: "Choose a faction as you play Merc Cruiser.\
                     Merc Cruiser has that faction.".to_string(),
                            config: Some(UserConfigMeta {
                                describe: Box::new(|i| match i {
                                    0 => "Mech faction",
                                    1 => "Fed faction",
                                    2 => "Blob faction",
                                    3 | _ => "Star faction",
                                }.to_string()),
                                config_method: ActionConfigMethod::Range(0, 3)
                            })
                        },
                        action: Box::new(|game, cfg| {
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
                                effects: Effects::new()
                            };
                            let id = game.get_current_player_mut().give_card_to_hand(card);
                            game.get_current_player_mut().plan_scrap(&id).unwrap();
                            Succeed
                        })
                    }
                })
            }
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
    PickTradeRowCards(u32, RelativePlayer),

    /// let the client choose one of the action config methods
    ExclusiveOr(Box<ActionConfigMethod>, Box<ActionConfigMethod>),

    /// must complete both action config methods
    Both(Box<ActionConfigMethod>, Box<ActionConfigMethod>)
}

impl ActionConfig {
    fn no_config() -> ActionConfig {
        ActionConfig {
            pre_config_meta: None
        }
    }
}

impl ActionExecution {
    pub fn no_config(&self) -> bool {
        self.config.is_none()
    }
}