use std::collections::HashMap;
use std::str::FromStr;
use crate::game::components::card::details::{CardSizeT, CardSource};
use crate::game::Player;

#[derive(Clone, Debug)]
pub enum ClientActionOptionQuery {
    /// requires the user to choose a card from the source
    CardSelection(CardSource)
}

#[derive(Debug)]
pub enum ClientActionOptionResponse {
    /// the card chosen from the source, the index given
    CardSelection(CardSource, CardSizeT)
}

pub struct ClientQuery {
    pub action_query: ClientActionOptionQuery,
    pub performer: Player
}

pub trait Client {
    /// function that should be able to answer these "action requests"
    /// that are configuration for an action
    fn resolve_action_query(&mut self, query: ClientQuery) -> ClientActionOptionResponse;
    fn alert<T>(&self, message: &HashMap<Player, &str>, interrupt: &HashMap<Player, Option<Vec<(&str, T)>>>) -> Option<T>;
}