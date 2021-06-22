extern crate yaml_rust;
extern crate regex;
use std::collections::{HashSet};
use std::fs;

use crate::game::components::card::{Base, Card};
use crate::game::components::faction::Faction;

use self::yaml_rust::{Yaml, YamlLoader};
use self::regex::Regex;
use crate::game::Goods;

pub fn parse_file (filepath: String) -> Result<Vec<Card>, &'static str> {
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
                                            Err(_) => return Err("card not parsed correctly")
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

    if let Some(mp) = obj["effects"].as_hash() {
        for (k, v) in mp {
            if let Some(ks) = k.as_str() {
                if let Some(vs) = v.as_str() {
                    effects.insert((ks.to_owned(), vs.to_owned()));
                } else {
                    return Err("value was not a string")
                }
            } else {
                return Err("key could not be a string")
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

pub fn parse_goods(good_str: &str) -> Option<Goods> {
    // Remember: C:A:T

    // also this has been known to be a bad design pattern but for now I'll keep it in here for simplicity
    // see https://docs.rs/regex/1.5.4/regex/
    let pattern = Regex::new(r"G(\d*):(\d*):(\d*)").unwrap();

    let caps = pattern.captures(good_str);
    match caps {
        None => None,
        Some(caps) => {
            if let Some(c) = caps.get(1) {
                if let Some(a) = caps.get(2) {
                    if let Some(t) = caps.get(3) {
                        Some(Goods {
                            // unwrapping because we used regex to get strings
                            combat: c.as_str().parse().unwrap(),
                            authority: a.as_str().parse().unwrap(),
                            trade: t.as_str().parse().unwrap()
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}