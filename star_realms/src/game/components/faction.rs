use crate::game::components::faction::Faction::*;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum Faction {
    Mech, // The Machine Cult
    Star, // The Star Empire
    Blob, // The Blobs
    Fed, // The Trade Federation
}

pub fn all_factions() -> Vec<Faction> {
    vec![Mech, Star, Blob, Fed]
}