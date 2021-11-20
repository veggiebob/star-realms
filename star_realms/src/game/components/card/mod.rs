use crate::game::components::faction::{Faction, all_factions};
use std::collections::{HashSet};
use crate::game::components::{Defense, Coin};
use crate::game::Goods;
use crate::parse::parse_goods;
use std::hash::{Hash, Hasher};
use crate::game::util::Join;

pub mod details;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Card {
    pub cost: Coin,
    pub name: String,
    pub base: Option<Base>, // None -> not a base, otherwise which base is it?
    pub synergizes_with: HashSet<Faction>,
    pub content: Join<PlaySet>
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Base {
    Outpost(Defense),
    Base(Defense)
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

impl Hash for Card {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}