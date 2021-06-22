use crate::game::components::faction::{Faction, all_factions};
use std::collections::{HashSet};
use crate::game::components::{Defense, Coin};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Card {
    pub cost: Coin,
    pub name: String,
    pub base: Option<Base>, // None -> not a base, otherwise which base is it?
    pub synergizes_with: HashSet<Faction>,
    pub effects: HashSet<(String, String)>
}

pub struct CardStatus {
    pub effects_used: HashSet<(String, String)>,
    pub scrapped: bool
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Base {
    Outpost(Defense),
    Base(Defense)
}

impl CardStatus {
    pub fn new () -> CardStatus {
        CardStatus {
            effects_used: HashSet::new(),
            scrapped: false
        }
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
    fn synergizes_over (&self, faction: &Faction) -> bool {
        self.synergizes_with.contains(faction)
    }
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
}