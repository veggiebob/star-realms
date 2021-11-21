use crate::game::components::Defense;
use crate::game::Goods;
use crate::game::util::Join;
use std::fmt::{Debug, Formatter};

pub type PlaySet = Vec<Play>;
pub type Play = (Option<Join<Requirement>>, Action);

type ConditionFunc = ();

#[derive(Clone)]
pub enum Requirement {
    Condition(ConditionFunc),
    Cost(Sacrifice)
}

#[derive(Debug, Clone)]
pub struct Sacrifice {
    pub scrap: bool,
    pub goods: Option<Goods>
}


type ActionFunc = ();


type ClientActionOptionQuery = ();

#[derive(Debug, Clone)]
pub enum Action {
    Sequential(Box<Join<Action>>, Box<Join<Action>>),
    Unit(Join<Actionable>)
}

#[derive(Debug, Clone)]
pub struct Actionable {
    run: ActionFunc,
    client_query: ClientActionOptionQuery
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

impl Debug for Requirement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Requirement::Condition(_) => f.debug_tuple("").finish(),
            Requirement::Cost(sacrifice) => sacrifice.fmt(f)
        }
    }
}
