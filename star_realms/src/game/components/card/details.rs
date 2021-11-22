use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::str::FromStr;

use crate::game::actions::client_comms::{ClientActionOptionQuery, ClientActionOptionResponse};
use crate::game::components::Defense;
use crate::game::components::Goods;
use crate::game::{GameState, RelativePlayer};
use crate::game::util::{Failure, Join};

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

    /// required to scrap this card
    ScrapThis,

    /// scrap some amount of cards, from discard and/or hand
    Scrap(CardSizeT, Join<CardSource>),

    /// required to spend the goods
    Goods(Goods),

    /// required to discard that many cards
    Discard(CardSizeT)

    // DiscardThis??
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum CardSource {
    Hand(RelativePlayer),
    Discard(RelativePlayer),
    Deck(RelativePlayer),
    TradeRow
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
/// a "fail" indicates an action that was valid but had an error during it,
/// so the exhaustion should be reset
type ActionFunc = dyn FnMut(&mut GameState, ActionConfig) -> ActionResult;

type ActionConfig = Option<ClientActionOptionResponse>;
type ActionResult = Result<(), String>;

#[derive(Debug, Clone)]
pub enum Action {
    Sequential(Box<Join<Action>>, Box<Join<Action>>),
    Unit(Join<Actionable>)
}

#[derive(Clone)]
pub struct Actionable {

    /// function that operates on game data when this action is chosen
    /// by the client
    pub run: Rc<Box<ActionFunc>>,

    /// some data structure representing a request for a decision from the client
    /// this will be blocking
    /// in the case where there is no query, just run the action
    pub client_query: Option<Join<ClientActionOptionQuery>>
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

impl Actionable {
    pub fn no_args<F>(f: F) -> Actionable
        where F: 'static + FnMut(&mut GameState, ActionConfig) -> ActionResult {
        Actionable {
            client_query: None,
            run: Rc::new(Box::new(f))
        }
    }
    pub fn new<F>(query: Join<ClientActionOptionQuery>, f: F) -> Actionable
        where F: 'static + FnMut(&mut GameState, ActionConfig) -> ActionResult {
        Actionable {
            client_query: Some(query),
            run: Rc::new(Box::new(f))
        }
    }
}

impl Debug for Actionable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.client_query.fmt(f)
    }
}
