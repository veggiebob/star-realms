#[macro_use] extern crate maplit;

pub mod game;
pub mod resources;
mod parse;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::mem;

    use yaml_rust::yaml::Yaml::Hash;
    use yaml_rust::YamlLoader;

    use crate::game::{GameState, PlayerArea, RelativePlayer};
    use crate::game::actions::{add_goods, draw_card, scrap_card, specially_place_next_acquired};
    use crate::game::card_library::CardLibrary;
    use crate::game::components::card::Card;
    use crate::game::components::card::details::{Action, Base, Exhaustibility, Play, Requirement, Sacrifice, CardSource, Actionable};
    use crate::game::components::faction::Faction;
    use crate::game::components::Goods;
    use crate::game::components::stack::{SimpleStack, Stack};
    use crate::game::util::{Join, Named};
    use crate::parse::{parse_card, parse_file, parse_goods};
    use crate::game::requirements::synergy;

    #[test]
    fn test_shuffle() {
        print_long_message("testing shuffle");
        for _ in 0..10 {
            let mut stack = SimpleStack::new((1..5).collect());
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
    fn join_playground() {
        let joined = Join::Unit(5);
        let v = vec![1, 2, 3, 4, 5];
        let joined_2 = Join::union(v);
        println!("{:?}", if let Join::Union(xs) = joined_2 {
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
