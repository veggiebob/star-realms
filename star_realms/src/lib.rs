#![allow(dead_code)]
pub mod game;
mod parse;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use yaml_rust::yaml::Yaml::Hash;
    use yaml_rust::YamlLoader;

    use crate::game::components::card::details::{Base, Action, Requirement, Sacrifice};
    use crate::game::components::card::Card;
    use crate::game::components::faction::Faction;
    use crate::game::components::stack::Stack;
    use crate::game::{Goods, GameState, PlayerArea};
    use crate::parse::{parse_card, parse_file, parse_goods};
    use crate::game::card_library::CardLibrary;
    use std::mem;
    use crate::game::util::Join;
    use crate::game::actions::{add_goods, draw_card};

    #[test]
    fn test_shuffle() {
        print_long_message("testing shuffle");
        for _ in 0..10 {
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
    
    #[test]
    fn test_sizes () {
        println!("Size of String: {}", mem::size_of::<String>());
        println!("Size of CardLibrary: {}", mem::size_of::<CardLibrary>());
        println!("Size of GameState: {}", mem::size_of::<GameState>());
        println!("Size of Card: {}", mem::size_of::<Card>());
        println!("Size of &Card: {}", mem::size_of::<&Card>());
        println!("Size of PlayerArea: {}", mem::size_of::<PlayerArea>());
        println!("Size of HashSet<(String, String)>: {}", mem::size_of::<HashSet<(String, String)>>());
    }

    #[test]
    fn card_test_1() {
        let card = Card {
            cost: 3,
            name: "Outland Station".to_string(),
            base: Some(Base::Base(4)),
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Fed);
                set
            },
            content: Some(vec![
                (
                    None,
                    Action::Unit(
                        Join::choose(vec![
                            add_goods(Goods { trade: 1, authority: 0, combat: 0 }),
                            add_goods(Goods { trade: 0, authority: 3, combat: 0 })
                        ])
                    )
                ),
                (
                    Some(
                        Join::Unit(
                            Requirement::Cost(
                                Sacrifice { scrap: true, goods: None }
                            )
                        )
                    ),
                    Action::Unit(Join::Unit(draw_card()))
                )
            ])
        };
    }

    #[test]
    fn join_playground() {
        let joined = Join::Unit(5);
        let v = vec![1, 2, 3, 4, 5];
        let joined_2 = Join::all(v);
        println!("{:?}", if let Join::All(xs) = joined_2 {
            xs
        } else {
            vec![]
        })
    }

    fn print_long_message(msg: &str) {
        println!();
        println!("==================={}===================", msg);
    }
}
