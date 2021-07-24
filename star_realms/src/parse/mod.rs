extern crate yaml_rust;
extern crate regex;
use std::collections::{HashSet, HashMap};
use std::fs;

use crate::game::components::card::{Base, Card, Effects, EffectConfig, EffectConfigPair};
use crate::game::components::faction::Faction;

use self::yaml_rust::{Yaml, YamlLoader};
use self::regex::Regex;
use crate::game::Goods;
use crate::game::components::Coin;
use self::yaml_rust::yaml::Hash;

pub fn parse_file (filepath: String) -> Result<Vec<Card>, String> {
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
                                    None => return Err("key is not a string".to_string()),
                                    Some(key) => {
                                        match parse_card(key, v.clone()) {
                                            Ok(nice) => cards.push(nice),
                                            Err(e) => return Err(
                                                format!("error on card '{}': {}", key, e))
                                        }
                                    }
                                }
                            }
                            Ok(cards)
                        }
                        _ => return Err("must be a hash".to_string())
                    }
                }
                Err(e) => return Err(format!("scan error: {}", e))
            }
        }
        Err(e) => return Err(format!("error reading file: {}", e))
    }
}

/*
Example card:

card_name:
    base: true|false
    [defense: <u8>]
    [outpost: true|false]
    cost: <u8>
    [synergy:
        - m|f|t|s|b
        - m|f|t|s|b
        ...
        ]
    [effects:
        - <cond1>: <actn1>
        - <cond2>: <actn2>
        ...
        ]
    [effects_config:
        - actn:
            base: <actn>
            <k1>: <v1>
            <k2>: <v2>
            ...
          cond:
            base: <cond>
            <k1>: <v1>
            <k2>: <v2>
            ...
        - actn:
            base: <actn>
            <k1>: <v1>
            ...
          cond:
            base: <cond>
            <k1>: <v1>
            ...
    ]

 */
pub fn parse_card (name: &str, yaml: Yaml) -> Result<Card, String> {
    let obj = yaml;
    let base = match obj["base"].as_bool() {
        Some(_base) => match _base {
            true => {
                let defense = match obj["defense"].as_i64() {
                    Some(_defense) => _defense as u8,
                    None => return Err("must supply a 'defense' (int) value if 'base' is true".to_string())
                };
                match obj["outpost"].as_bool() {
                        Some(_outpost) => match _outpost {
                            true => Some(Base::Outpost(defense)),
                            false => Some(Base::Base(defense))
                        }
                        None => return Err("must supply an 'outpost' (bool) value if 'base' is true".to_string())
                    }
                }
            false => None
        }
        None => return Err("must supply 'base'".to_string())
    };

    let cost = match obj["cost"].as_i64() {
        Some(_cost) if (0 <= _cost && _cost <= 255) => _cost as Coin,
        Some(x) => return Err(format!("{} is not in the range 0..255 for coins", x)),
        None => return Err("must supply 'cost'".to_string())
    };

    let mut synergizes_with = HashSet::new();
    let mut effects = HashSet::new();

    // no synergy is ok, some cards don't have it
    // but if synergy is provided and it's not a vec, it's bad
    if !obj["synergy"].is_badvalue() {
        if let None = obj["synergy"].as_vec() {
            return Err("synergy must be a vec".to_string())
        }
    }

    if let Some(synergy) = obj["synergy"].as_vec() {
        for syn in synergy {
            if let Some(syn) = syn.as_str() {
                match syn {
                    "m" => synergizes_with.insert(Faction::Mech),
                    "s" => synergizes_with.insert(Faction::Star),
                    "b" => synergizes_with.insert(Faction::Blob),
                    "f" | "t" => synergizes_with.insert(Faction::Fed), // f or t for Trade Federation
                    _ => return Err(format!("synergy symbol '{}' was not one of [m, s, b, f]", syn)),
                };
            } else {
                return Err("synergy could not be a string".to_string())
            }
        }
    }

    if let Some(mp) = obj["effects"].as_vec() {
        for yaml in mp {
            if let Yaml::Hash(ks) = yaml {
                for (k, v) in ks {
                    if let Some(k) = k.as_str() {
                        if let Some(v) = v.as_str() {
                            effects.insert((k.to_string(), v.to_string()));
                        } else {
                            return Err("value of effect could not be a string".to_string())
                        }
                    } else {
                        return Err("key of effect could not be a string".to_string());
                    }
                }
            } else {
                return Err("key could not be a string".to_string())
            }
        }
    }

    let mut effect_config = HashMap::new();
    if let Some(cfgs) = obj["effects_config"].as_vec() {
        for cfg in cfgs {
            let (k, v) = parse_effect_config(name, cfg)?;
            effect_config.insert(k, v);
        }
    }

    Ok(Card {
        cost,
        name: name.to_owned(),
        base,
        synergizes_with,
        effects: {
            let mut tmp = Effects::from_no_config_effects(effects);
            tmp.add_configs(effect_config);
            tmp
        }
    })
}

type KeyedEffectConfig = ((String, String), EffectConfigPair);
fn parse_effect_config(card_name: &str, config: &Yaml) -> Result<KeyedEffectConfig, String> {
    let cond_key = Yaml::String("cond".to_string());
    let actn_key = Yaml::String("actn".to_string());
    if let Yaml::Hash(kvs) = config {
        if kvs.contains_key(&cond_key) {
            if kvs.contains_key(&actn_key) {
                if let Yaml::Hash(cond_cfg) = kvs.get(&cond_key).unwrap() {
                    if let Yaml::Hash(actn_cfg) = kvs.get(&actn_key).unwrap() {
                        todo!("something to do with parse_action_config and parse_cond_config")
                    } else {
                        Err(format!(
                            "'{}' needs an object for the 'actn' key",
                            &card_name
                        ))
                    }
                } else {
                    Err(format!(
                        "'{}' needs an object for the 'cond' key",
                        &card_name
                    ))
                }
            } else {
                Err(format!(
                    "'{}' has a misformatted effect config entry. It doesn't contain the 'actn' key",
                    &card_name
                ))
            }
        } else {
            Err(format!(
                "'{}' has a misformatted effect config entry. It doesn't contain the 'cond' key",
                &card_name))
        }
    } else {
        Err(
            format!(
                "'{}' has a misformatted effect config entry. Is it an anonymous object?",
                &card_name))
    }
}

fn parse_action_config(card_name: &str, hash: Hash) -> Result<(String, EffectConfig), String> {
    /*
    actn:
        base: <actn>
        <k1>: <v1>
        <k2>: <v2>
        ...
     */
    todo!()
}
fn parse_cond_config(card_name: &str, hash: Hash) -> Result<(String, EffectConfig), String> {
    /*
    cond:
        base: <cond>
        <k1>: <v1>
        <k2>: <v2>
        ...
     */
    todo!()
}

/// example: G0.0.1
pub fn parse_goods(good_str: &str) -> Option<Goods> {
    // Remember: C.A.T

    // also this has been known to be a bad design pattern but for now I'll keep it in here for simplicity
    // see https://docs.rs/regex/1.5.4/regex/
    let pattern = Regex::new(r"G(\d+).(\d+).(\d+)").unwrap();

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