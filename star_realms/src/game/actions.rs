use crate::game::components::Goods;
use crate::game::components::card::details::{Actionable, CardSource};
use crate::game::util::Join;
use std::collections::HashSet;
use crate::game::util::Failure::Succeed;
use crate::game::GameState;

pub fn add_goods(goods: Goods) -> Actionable {
    todo!()
}

pub fn draw_card() -> Actionable {
    Actionable::no_args(|game: &mut GameState| {
        game.get_current_player_mut().draw_card_into_hand()
    })
}

pub fn scrap_card(source: HashSet<CardSource>) -> Actionable {
    todo!()
}

/// "put the next ship or base you acquire this turn into your "[destination]"
pub fn specially_place_next_acquired(destination: CardSource) -> Actionable {
    todo!()
}