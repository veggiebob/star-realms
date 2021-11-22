use std::collections::HashMap;
use std::str::FromStr;
use crate::game::components::card::details::{CardSizeT, CardSource};

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