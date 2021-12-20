use crate::game::components::faction::Faction;
use crate::game::components::card::details::{Requirement, Sacrifice};
use crate::game::components::Goods;

// requires access to the player's hand
// and player's turn data
pub fn synergy(faction: Faction) -> Requirement {
    // todo: still implement this pls
    Requirement::Cost(Sacrifice::Goods(Goods::none()))
}