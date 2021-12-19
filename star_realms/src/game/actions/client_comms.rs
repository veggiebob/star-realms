use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use crate::game::components::card::details::{CardSizeT, CardSource};
use crate::game::Player;
use crate::game::components::card::CardRef;

/// Query descriptors that require the client to get data from the user.
/// Each of the following should have a corresponding response in the
/// ClientActionOptionResponse enum.
// note to self: this is employed by descriptions in the card itself,
// so any amount of joining or permutations of each query have been taken care of
#[derive(Clone, Debug)]
pub enum ClientActionOptionQuery {
    /// Requires the user to choose a card from the source
    /// Expected response: CardSelection
    CardSelection(CardSource),
}

/// represents a filter for cards (returning true = fits the restriction)
pub struct Restriction(Box<dyn Fn(CardRef) -> bool>);

#[derive(Debug)]
pub enum ClientActionOptionResponse {
    /// the card chosen from the source, the index given
    CardSelection(CardSource, CardSizeT)
}

/// umbrella query that the client receives
pub struct ClientQuery {
    pub action_query: ClientActionOptionQuery,
    pub performer: Player
}

pub struct StyledText {
    pub style: TextStyle,
    pub text: String
}

impl From<String> for StyledText {
    fn from(s: String) -> Self {
        StyledText {
            style: TextStyle::plain(),
            text: s
        }
    }
}

/// Homemade styling system that can be abstracted over simple
/// text, or HTML, markdown, etc. depending on the implementation.
/// It can also be ignored without any problems.
/// (presently has no implementation, but exists for future finishing touches)
pub struct TextStyle();

impl TextStyle {
    fn new() -> TextStyle {
        TextStyle()
    }
    pub fn plain() -> TextStyle {
        TextStyle::new()
    }
    pub fn attention() -> TextStyle {
        TextStyle::new()
    }
    pub fn warn() -> TextStyle {
        TextStyle::new()
    }
    pub fn error() -> TextStyle {
        TextStyle::new()
    }
    pub fn success() -> TextStyle {
        TextStyle::new()
    }
    pub fn faint() -> TextStyle {
        TextStyle::new()
    }
}

pub trait Client {
    /// function that should be able to answer these "action requests"
    /// that are configuration for an action
    fn resolve_action_query(&mut self, query: ClientQuery) -> ClientActionOptionResponse;

    /// a generic way to send messages in text format to the client
    fn alert<T>(&self,
                message: &HashMap<Player, &str>,
                interrupt: &HashMap<Player, Option<Vec<(&str, T)>>>,
                style: TextStyle) -> Option<T>;
}