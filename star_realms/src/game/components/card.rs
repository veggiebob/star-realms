use crate::game::components::faction::{Faction, all_factions};
use std::collections::{HashSet, HashMap};
use crate::game::components::{Defense, Coin};
use crate::game::Goods;
use crate::game::effects::{is_free_cond, PreConfig};
use std::hash::{Hash, Hasher};
use std::collections::hash_set::Iter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Card {
    pub cost: Coin,
    pub name: String,
    pub base: Option<Base>, // None -> not a base, otherwise which base is it?
    pub synergizes_with: HashSet<Faction>,
    pub effects: Effects // relational structure
}

#[derive(Debug)]
pub struct CardStatus {
    pub in_play: bool,
    pub effects_used: HashSet<(String, String)>,
    pub scrapped: bool
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Base {
    Outpost(Defense),
    Base(Defense)
}

impl CardStatus {
    pub fn new() -> CardStatus {
        CardStatus {
            in_play: false,
            effects_used: HashSet::new(),
            scrapped: false
        }
    }
    pub fn reveal(&mut self) {
        self.in_play = true;
    }
    pub fn all_effects_used(&self, card: &Card) -> bool {
        self.unused_effects(card).is_empty()
    }
    pub fn unused_effects(&self, card: &Card) -> HashSet<(String, String)> {
        let mut eff = HashSet::new();
        for e in card.effects.iter() {
            if !self.effects_used.contains(e) {
                eff.insert(e.clone());
            }
        }
        eff
    }

    pub fn is_free(cond: &String) -> bool {
        is_free_cond(cond)
    }

    /// protocol for resetting base after a turn is over when it isn't destroyed
    pub fn reset_base(&mut self) {
        self.effects_used.clear();
        // we don't take it "out of play" because it's still revealed
    }

    pub fn use_effect(&mut self, effect: &(String, String)) {
        self.reveal();
        self.effects_used.insert(effect.clone());
    }
}

impl Base {
    pub fn is_outpost (&self) -> bool {
        match self {
            Base::Outpost(_) => true,
            _ => false
        }
    }
}

impl Card {

    /// will this card provide synergy for `faction`
    fn synergizes_over (&self, faction: &Faction) -> bool {
        self.synergizes_with.contains(faction)
    }

    /// the set of factions that this card synergizes with the `other` card
    fn synergizes_with (&self, other: Card) -> HashSet<Faction> {
        let mut set = HashSet::new();
        let factions = all_factions();
        for f in factions {
            if other.synergizes_over(&f) && self.synergizes_over(&f) {
                set.insert(f);
            }
        }
        set
    }

    /// will this card synergize with `other`
    fn does_synergize (&self, other: Card) -> bool {
        self.synergizes_with(other).len() > 0
    }
}

impl Hash for Card {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// essentially a constant depth JSON object
pub type EffectConfig = HashMap<String, String>;

/// a pair representing a config for a (condition, action)
pub type EffectConfigPair = (EffectConfig, EffectConfig);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effects {
    /// cond-actn pairs that contain all of the effects
    effects: HashSet<(String, String)>,
    /// cond-actn pairs that serve as keys for their configs
    configs: HashMap<(String, String), EffectConfigPair>
}

impl Effects {
    pub fn new() -> Effects {
        Effects {
            effects: HashSet::new(),
            configs: HashMap::new()
        }
    }
    pub fn from_no_config_effects(effects: HashSet<(String, String)>) -> Effects {
        Effects {
            effects,
            configs: HashMap::new()
        }
    }
    pub fn get_all_effects(&self) -> &HashSet<(String, String)> {
        return &self.effects;
    }
    pub fn add(&mut self, kv: (String, String)) {
        self.effects.insert(kv);
    }
    pub fn add_effects(&mut self, kvs: Iter<(String, String)>) {
        for kv in kvs {
            self.add(kv.clone());
        }
    }
    pub fn add_config(&mut self, key: (String, String), properties: EffectConfigPair) {
        self.configs.insert(key, properties);
    }
    pub fn add_configs(&mut self, configs: HashMap<(String, String), EffectConfigPair>) {
        for (k, v) in configs {
            self.add_config(k, v);
        }
    }
    pub fn get_config(&self, key: &(String, String)) -> Option<&EffectConfigPair> {
        self.configs.get(key)
    }
    pub fn get_action_pre_config(&self, cond: &String, actn: &String) -> Option<PreConfig> {
        let cfg: Option<&(EffectConfig, EffectConfig)> = self.get_config(&(cond.clone(), actn.clone()));
        cfg.and_then(|cfg| Some(PreConfig::create(cfg.clone().0)))
    }
    pub fn iter(&self) -> Iter<(String, String)> {
        self.effects.iter()
    }
}