#![allow(dead_code)]

mod game;
mod parse;

#[cfg(test)]
mod tests {
    use crate::game::{Stack, Goods, validate_card};
    use crate::parse::{parse_file, parse_card, parse_goods};
    use yaml_rust::{YamlLoader};
    use yaml_rust::yaml::Yaml::Hash;
    use crate::game::components::card::Card;
    use crate::game::components::card::Base;
    use std::collections::HashSet;
    use crate::game::components::faction::Faction;

    #[test]
    fn test_shuffle() {
        print_long_message("testing shuffle");
        for i in 0..10 {
            let mut stack = Stack::new((1..5).collect());
            stack.shuffle();
            println!("{:?}", stack);
        }
    }

    #[test]
    fn test_yaml() {
        // yeah: don't test your libraries, but also idk how this works
        // so this will be left as an example for myself
        print_long_message("yaml test");
        let yaml = YamlLoader::load_from_str("\
card1:
  base: false
  name: bazinga
card2:
  base: false
  name: card2
        ");
        let yaml = &yaml.expect("not parsed correctly")[0];
        println!("{:?}", yaml);
        match yaml {
            Hash(b) => {
                for (k, v) in b {
                    println!("{:?} -> {:?}", k.as_str().unwrap(), v);
                }
            }
            _ => panic!("not a hash???")
        }
    }

    #[test]
    fn parse_yaml_card1() {
        // print_long_message("testing card 1");
        let yaml = YamlLoader::load_from_str("\
card1:
  base: false
  synergy:
    - m
    - b
        ");
        let yaml = &yaml.unwrap()[0];
        let card = parse_card("card1", yaml["card1"].clone()).unwrap();
        // println!("{:?}", card);
        assert_eq!(card, Card {
            name: "card1".to_owned(),
            base: None,
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Mech);
                set.insert(Faction::Blob);
                set
            },
            effects: HashSet::new(),
        })
    }

    #[test]
    fn parse_yaml_card2() {
        // print_long_message("testing card 2");
        let yaml = YamlLoader::load_from_str("\
card2:
  base: true
  defense: 4
  outpost: true
  synergy:
    - s
    - f
  effects:
    any: test
        ");
        let yaml = &yaml.unwrap()[0];
        let card = parse_card("card2", yaml["card2"].clone()).unwrap();
        println!("{:?}", card);
        assert_eq!(card, Card {
            name: "card2".to_owned(),
            base: Some(Base::Outpost(4)),
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Star);
                set.insert(Faction::Fed);
                set
            },
            effects: {
                let mut set = HashSet::new();
                set.insert(("any".to_owned(), "test".to_owned()));
                set
            },
        });
        assert!(validate_card(&card));
    }

    #[test]
    fn parse_multiple_cards() {
        let cards = parse_file("config/test.yaml".to_owned()).unwrap();
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0], Card {
            name: "card1".to_owned(),
            base: None,
            effects: HashSet::new(),
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Mech);
                set.insert(Faction::Fed);
                set
            },
        });
        assert_eq!(cards[1], Card {
            name: "card2".to_owned(),
            base: Some(Base::Base(6)),
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Star);
                set.insert(Faction::Blob);
                set
            },
            effects: {
                let mut set = HashSet::new();
                set
            },
        });
    }

    #[test]
    fn test_parse_goods () {
        assert_eq!(parse_goods("G6:3:0").unwrap(), Goods {
            combat: 6,
            authority: 3,
            trade: 0
        });
        assert_eq!(parse_goods("G12:0:4").unwrap(), Goods {
            combat: 12,
            authority: 0,
            trade: 4
        });
        assert_eq!(parse_goods("G144:225:124").unwrap(), Goods {
            combat: 144,
            authority: 225,
            trade: 124
        });
    }

    fn print_long_message(msg: &str) {
        println!();
        println!("==================={}===================", msg);
    }
}
