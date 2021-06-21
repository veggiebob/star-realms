extern crate yaml_rust;
use crate::game::components::card::{Card, Base};
use self::yaml_rust::{YamlLoader, ScanError, Yaml};
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Add;
use crate::game::components::faction::Faction;

pub fn parse_file (filepath: &String) -> Result<Vec<Card>, &str> {
    let contents = fs::read_to_string(filepath);
    match contents {
        Ok(contents) => {
            let yaml = YamlLoader::load_from_str(&*contents);
            match yaml {
                Ok(yaml) => {
                    let yaml = &yaml[0];
                    match yaml {
                        Yaml::Hash(b) => {
                            let mut cards = vec![];
                            for (k, v) in b {
                                let key = k.as_str();
                                match key {
                                    None => return Err("key is not a string"),
                                    Some(key) => {
                                        match parse_card(key, v.clone()) {
                                            Ok(nice) => cards.push(nice),
                                            Err(e) => return Err("card not parsed correctly")
                                        }
                                    }
                                }
                            }
                            Ok(cards)
                        }
                        _ => return Err("must be a hash")
                    }
                }
                Err(_) => return Err("scan error")
            }
        }
        Err(_) => return Err("reading file error")
    }
}

pub fn parse_card (name: &str, yaml: Yaml) -> Result<Card, &'static str> {
    let obj = yaml;
    let base = match obj["base"].as_bool() {
        Some(_base) => match _base {
            true => {
                let defense = match obj["defense"].as_i64() {
                    Some(_defense) => _defense as u8,
                    None => return Err("must supply a 'defense' (int) value if 'base' is true")
                };
                match obj["outpost"].as_bool() {
                        Some(_outpost) => match _outpost {
                            true => Some(Base::Outpost(defense)),
                            false => Some(Base::Base(defense))
                        }
                        None => return Err("must supply an 'outpost' (bool) value if 'base' is true")
                    }
                }
            false => None
        }
        None => return Err("must supply 'base'")
    };
    let mut synergizes_with = HashSet::new();
    let mut effects = HashSet::new();

    // no synergy is ok, some cards don't have it
    // but if synergy is provided and it's not a vec, it's bad
    if !obj["synergy"].is_badvalue() {
        if let None = obj["synergy"].as_vec() {
            return Err("synergy must be a vec")
        }
    }

    if let Some(synergy) = obj["synergy"].as_vec() {
        for syn in synergy {
            if let Some(syn) = syn.as_str() {
                match syn {
                    "m" => synergizes_with.insert(Faction::Mech),
                    "s" => synergizes_with.insert(Faction::Star),
                    "b" => synergizes_with.insert(Faction::Blob),
                    "f" => synergizes_with.insert(Faction::Fed),
                    _ => return Err("synergy symbol was not one of {m, s, b, f}")
                };
            } else {
                return Err("synergy could not be a string")
            }
        }
    }
    Ok(Card {
        name: name.to_owned(),
        base,
        synergizes_with,
        effects,
    })
}