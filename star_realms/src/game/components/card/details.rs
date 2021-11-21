use crate::game::components::Defense;
use crate::game::components::Goods;
use crate::game::util::Join;
use std::fmt::{Debug, Formatter};


/// number type for counting cards
pub type CardSizeT = u32;

/// a vector of `Play`s
pub type PlaySet = Vec<Play>;

/// struct that contains a condition, an action, and an exhaustion rule
#[derive(Debug)]
pub struct Play {

    /// the condition of this play (None meaning free, otherwise
    /// some collection of requirements)
    pub cond: Option<Join<Requirement>>,

    /// the action of this play (either sequential or unit)
    pub actn: Action,

    /// how many times this play can be executed per turn
    pub exhaust: Exhaustibility
}

#[derive(Debug, Clone)]
pub enum Exhaustibility {
    Once,
    UpTo(u32),
    Exactly(u32)
}

#[derive(Clone)]
pub enum Requirement {
    Condition(ConditionFunc),
    Cost(Sacrifice)
}

#[derive(Debug, Clone)]
pub enum Sacrifice {
    ScrapThis,

    /// scrap some amount of cards, from discard and/or hand
    Scrap(CardSizeT, Join<CardSource>),
    Goods(Goods),
    Discard(CardSizeT)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardSource {
    Hand,
    Discard
}

/*
access requirements:
your* hand (for synergy)
your* turn events (see root-level notes file) (ex. did you play a base this turn?)

 */
/// function that determines whether an action can or is allowed to be run
/// should be pure!
type ConditionFunc = ();


/*

access requirements:
your* hand
your* discard pile
your* turn events (see root-level notes file)
opponent* hand
opponent* free area (destroying target base)
trade row


 */
/// function that alters game data
type ActionFunc = ();

type ClientActionOptionQuery = ();

#[derive(Debug, Clone)]
pub enum Action {
    Sequential(Box<Join<Action>>, Box<Join<Action>>),
    Unit(Join<Actionable>)
}

#[derive(Debug, Clone)]
pub struct Actionable {

    /// function that operates on game data when this action is chosen
    /// by the client
    run: ActionFunc,

    /// some data structure representing a request for a decision from the client
    /// this will be blocking
    /// in the case where there is no query, just run the action
    client_query: Option<ClientActionOptionQuery>
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
