use crate::game::components::faction::Faction::*;
use std::str::FromStr;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum Faction {
    Mech, // The Machine Cult
    Star, // The Star Empire
    Blob, // The Blobs
    Fed, // The Trade Federation
}

impl FromStr for Faction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "m" => Ok(Faction::Mech),
            "s" | "y" => Ok(Faction::Star),
            "b" => Ok(Faction::Blob),
            "f" | "t" => Ok(Faction::Fed),
            _ => Err("Not one of [m, s, y, b, f, t]".to_string())
        }
    }
}

pub fn all_factions() -> Vec<Faction> {
    vec![Mech, Star, Blob, Fed]
}