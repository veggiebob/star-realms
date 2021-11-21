use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use details::Base;

use crate::game::components::{Coin, Defense};
use crate::game::components::faction::{all_factions, Faction};
use crate::game::Goods;
use crate::game::util::Join;
use crate::parse::parse_goods;
use crate::game::components::card::details::PlaySet;

pub mod details;

#[derive(Clone, Debug)]
pub struct Card {
    pub cost: Coin,
    pub name: String,
    pub base: Option<Base>, // None -> not a base, otherwise which base is it?
    pub synergizes_with: HashSet<Faction>,
    pub content: Option<PlaySet>
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

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name &&
            self.cost == other.cost &&
            self.base == other.base &&
            self.synergizes_with == other.synergizes_with
    }
}

impl Eq for Card {

}
