use std::collections::{HashMap, HashSet};

use client_comms::ClientActionOptionQuery;

use crate::game::components::card::details::{Actionable, CardSource};
use crate::game::components::Goods;
use crate::game::GameState;
use crate::game::util::Failure::Succeed;
use crate::game::util::Join;
use crate::game::actions::client_comms::ClientActionOptionResponse;

pub mod client_comms;

const CONFIG_REQUIRED: &str = "Client Error: Action.run: config is not present";
const WRONG_CONFIG: &str = "Client Erorr: Action.run: config is of the wrong type and was not checked before hand!";

/// add goods to the player
pub fn add_goods(goods: Goods) -> Actionable {
    Actionable::no_args(move |game, _| {
        game.get_current_player_mut().current_goods += goods;
        Succeed.as_result()
    })
}

/// draw one card from the deck
pub fn draw_card() -> Actionable {
    Actionable::no_args(|game: &mut GameState, _| {
        game.get_current_player_mut().draw_card_into_hand().as_result()
    })
}

/// scrap one card from any of the sources
pub fn scrap_card(sources: HashSet<CardSource>) -> Actionable {
    let mut joined = Vec::new();
    for source in sources {
        joined.push(ClientActionOptionQuery::CardSelection(source));
    }
    Actionable::new(Join::choose(joined), |game, config| {
        let cfg = config.ok_or_else(|| CONFIG_REQUIRED.to_string())?;

        if let ClientActionOptionResponse::CardSelection(source, index) = cfg {
            let mut stack = game.get_stack_mut(source);
            match stack.remove(index as usize) {
                None => Err(format!("Accessing {:?} at index {} is out of bounds", source, index)),
                Some(card) => {
                    game.scrapped.add(card);
                    Ok(())
                }
            }
        } else {
            Err(format!("{:?} caused -> {}", cfg, WRONG_CONFIG))
        }
    })
}

/// "put the next ship or base you acquire this turn into your "[destination]"
pub fn specially_place_next_acquired(destination: CardSource) -> Actionable {
    todo!()
}