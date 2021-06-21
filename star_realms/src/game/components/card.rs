use crate::game::components::faction::{Faction, all_factions};
use std::collections::{HashSet, HashMap};
use crate::game::GameState;
use std::iter::{FromIterator};

type Defense = u8;
type Coin = u8;
type Authority = u8;
type Combat = u8;

#[derive(Clone, Debug)]
pub struct Card {
    pub name: String,
    pub base: Option<Base>, // None -> not a base, otherwise which base is it?
    pub synergizes_with: HashSet<Faction>,
    pub effects: HashSet<String>,
}

#[derive(Clone, Debug)]
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

// things to do on the card:
struct Good {
    trade: Coin,
    authority: Authority,
    combat: Combat
}

enum Either<L, R> {
    Left(L),
    Right(R)
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

trait Predicate<T> {
    fn test(object: T) -> bool;
}