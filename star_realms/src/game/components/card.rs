use crate::game::components::faction::{Faction, all_factions};
use std::collections::HashSet;

type Defense = u8;
type Coin = u8;
type Authority = u8;
type Combat = u8;

type Synergizing = Box<dyn Fn(&Card, &Faction) -> bool>;
pub struct Card {
    base: Option<Base>, // None -> not a base, otherwise which base is it?
    synergizing_strategy: Synergizing,
}

pub fn synergizes_all() -> Synergizing {
    Box::new(|_, _| true)
}
pub fn synergizes_one(a: Faction) -> Synergizing {
    Box::new(move |_, f| *f == a)
}

enum Base {
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
        (self.synergizing_strategy)(self, faction)
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