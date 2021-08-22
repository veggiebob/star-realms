use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};
use std::rc::Rc;

use actions::ActionConfigMethod;
use ansi_term::Color;

use crate::game::{GameState, Goods, HandId, RelativePlayer};
use crate::game::components::card::{Base, Card, Effects};
use crate::game::components::faction::Faction;
use crate::game::RelativePlayer::Opponent;
use crate::game::util::Failure::{Fail, Succeed};
use crate::game::util::Failure;
use crate::parse::{try_parse_or};
use crate::game::effects::actions::get_action;

pub mod actions;
pub mod conditions;

// Effects!

/// A trait that the client implements for choosing configuration values for
pub trait ConfigSupplier {
    /// get a config value (u32) for an action based on this Config object
    fn get_config(&self, game: &GameState, config: &UserConfigMeta) -> u32;
}

/// I mean this is just what I use for error reporting in general
pub type ConfigError = String;

/// describes data contained in yaml configuration for action or condition
pub struct PreConfig {
    map: HashMap<String, String>,
    validated: bool
}

/// Describes how the action can be configured by the yaml
pub struct PreConfigMeta {
    pub required_keys: HashSet<String>,
    pub types: HashMap<String, PreConfigType>
}

/// Describes types that are allowed in yaml configuration
pub enum PreConfigType {
    Nat,
    String
}

/// Describes how the action is configured by the user
pub struct UserConfigMeta {
    /// dev-friendly description for each of the config values
    pub describe: Box<dyn Fn(u32) -> String>,
    /// enum that shows how to get the config value (u32)
    pub config_method: ActionConfigMethod
}

pub fn validate_condition(name: &String) -> Failure<String> {
    match conditions::get_condition(name.clone()) {
        Some(_) => Succeed,
        None => Fail(format!("Invalid condition: {}", name))
    }
}

pub fn validate_action(name: &String) -> Failure<String> {
    match get_action(name) {
        Some(_) => Succeed,
        None => Fail(format!("Invalid action: {}", name))
    }
}

pub fn validate_effect((cond, act): (&String, &String)) -> Failure<String> {
    match validate_condition(cond) {
        Succeed => match validate_action(act) {
            Succeed => Succeed,
            x => x
        },
        x => x,
    }
}

/// None -> valid
/// String -> invalid, with reason
pub fn validate_card_effects(card: &Card) -> Failure<String> {
    for (l, r) in card.effects.iter() {
        if let Fail(e) = validate_effect((l, r)) {
            return Fail(e)
        }
    }
    Succeed
}

pub fn assert_validate_card_effects(card: &Card) {
    if let Fail(e) = validate_card_effects(&card) {
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

impl PreConfig {
    pub fn create(map: HashMap<String, String>) -> PreConfig {
        PreConfig {
            map,
            validated: false
        }
    }
    pub fn validate (&self, meta: &PreConfigMeta) -> Failure<String> {
        for k in meta.required_keys.iter() {
            if !self.map.contains_key(k) {
                return Fail(format!("Requires the key '{}'", k));
            }
        }
        for (k, s) in self.map.iter() {
            if let Some(t) = meta.types.get(k) {
                return match t {
                    PreConfigType::Nat => try_parse_or::<u32>(
                        s,
                        format!(">>{}: {}<< must be of the Nat type", k, s)
                    ),
                    PreConfigType::String => try_parse_or::<u32>(
                        s,
                        format!(">>{}: {}<< must be of the String type", k, s)
                    )
                }
            }
        }
        Succeed
    }
    pub fn get_nat(&self, key: &str) -> u32 {
        self.map.get(key).unwrap().parse().unwrap()
    }
    pub fn get_str(&self, key: &str) -> &str {
        self.map.get(key).unwrap()
    }
}

impl PreConfigMeta {
    fn all_required(kvs: Vec<(&str, PreConfigType)>) -> PreConfigMeta {
        PreConfigMeta {
            required_keys: {
                let mut tmp = HashSet::new();
                for (key, _) in kvs.iter() {
                    tmp.insert(key.to_string());
                }
                tmp
            },
            types: {
                let mut tmp = HashMap::new();
                for (k,v) in kvs {
                    tmp.insert(k.to_string(), v);
                }
                tmp
            }
        }
    }
    fn optional_keys(&self) -> HashSet<&String> {
        return self.types.keys().filter(|k| self.required_keys.contains(*k)).collect();
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
