use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use crate::game::{GameState, Player};
use crate::game::components::card::{CardRef, Card};
use crate::game::components::card::details::{CardSizeT, CardSource};
use std::rc::Rc;

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

    /// Select a play from the list
    /// (outer list lining up with the stack of cards, inner list is plays attached to that card)
    /// Expected response: PlaySelection
    PlaySelection(Vec<Vec<String>>)
}

/// represents a filter for cards (returning true = fits the restriction)
pub struct Restriction(Box<dyn Fn(CardRef) -> bool>);

#[derive(Debug)]
pub enum ClientActionOptionResponse {
    /// the card chosen from the source, the index given
    CardSelection(CardSource, CardSizeT),

    /// the card index and play index selected from the choices.
    /// return None if there were no play choices
    PlaySelection(Option<(CardSizeT, CardSizeT)>)
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

impl From<&str> for StyledText {
    fn from(s: &str) -> Self {
        String::from(s).into()
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
    /// Function that should be able to answer these "action requests".
    /// That are configuration for an action.
    fn resolve_action_query(&mut self, query: ClientQuery, game: &GameState) -> ClientActionOptionResponse;

    /// A generic way to send messages and alerts in text format to the client.
    /// These would show up either as log output (if passive) or a pop-up dialog box in a GUI game
    fn alert<'a, T: Eq>(&self,
                message: &HashMap<Player, &str>,
                interrupt: &HashMap<Player, Option<Vec<(&str, &'a T)>>>,
                style: TextStyle) -> Option<&'a T>;

    fn message_player<T: Into<StyledText>>(&self, player: &Player, message: T) {
        let message = message.into();
        self.alert::<()>(
            &hashmap!{
                *player => message.text.as_str()
            },
            &all_players(None),
            message.style
        );
    }

    fn broadcast_message(&self, message: StyledText) {
        self.alert::<()>(
            &all_players(message.text.as_str()),
            &all_players(None),
            message.style);
    }

}

/// helper function for broadcast method
fn all_players<T: Clone>(item: T) -> HashMap<Player, T> {
    hashmap!{
            Player::Player1 => item.clone(),
            Player::Player2 => item
        }
}

/// handles the cache that is updated when the client receives updates
/// this only accounts for visible cards. I'd assume discard is also in this.
/// However, since it uses a generic map from CardSource, I assume it could be any updates
pub struct VisibleCardStackCache {
    cache: HashMap<CardSource, Vec<CardRef>>
}

impl VisibleCardStackCache {
    pub fn new() -> VisibleCardStackCache {
        VisibleCardStackCache {
            cache: HashMap::new()
        }
    }
    pub fn update<I: Iterator<Item=CardRef>>(&mut self, source: CardSource, cards: I) {
        let mut tmp = vec![];
        // does order need to be preserved?
        // I think iti does. If it were merely for the purposes of displaying, then no?
        for card in cards {
            tmp.push(card.clone());
        }
        self.cache.insert(source, tmp); // overwrite the stack completely
    }

    pub fn get(&self, source: CardSource) -> Option<Vec<CardRef>> {
        self.cache.get(&source).map(Clone::clone)
    }

    pub fn get_or_alert<C: Client>(&self, client: &C, source: CardSource) -> Option<Vec<CardRef>> {
        match self.get(source) {
            None => {
                client.broadcast_message(StyledText {
                    text: format!("Unable to get cached card deck {:?}", source),
                    style: TextStyle::error()
                });
                None
            }
            Some(deck) => Some(deck)
        }
    }
}
