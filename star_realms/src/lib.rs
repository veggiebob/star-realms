#![allow(dead_code)]
mod game;
mod parse;

#[cfg(test)]
mod tests {
    use crate::game::Stack;
    use crate::parse::{parse_file, parse_card};
    use yaml_rust::{YamlLoader, Yaml};
    use yaml_rust::yaml::Yaml::Hash;

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
        println!("og: ");
        println!("{:?}", yaml);
        println!("iterated: ");
        match yaml {
            Hash(b) => {
                for (k, v) in b {
                    println!("{:?} -> {:?}", k.as_str().unwrap(), v);
                }
            },
            _ => panic!("not a hash???")
        }
    }

    #[test]
    fn parse_yaml_card1() {
        let yaml = YamlLoader::load_from_str("\
card1:
  base: false
  synergy:
    - m
    - b
        ");
        let yaml = &yaml.unwrap()[0];
        let card = parse_card("card1", yaml["card1"].clone()).unwrap();
        println!("{:?}", card);
    }

    #[test]
    fn parse_yaml_card2() {
        let yaml = YamlLoader::load_from_str("\
card2:
  base: true
  defense: 4
  outpost: true
  synergy:
    - s
    - f
        ");
        let yaml = &yaml.unwrap()[0];
        let card = parse_card("card2", yaml["card2"].clone()).unwrap();
        println!("{:?}", card);
    }

    fn print_long_message(msg: &str) {
        println!();
        println!("==================={}===================", msg);
    }
}
