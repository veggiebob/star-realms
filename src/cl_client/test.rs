use crate::cl_client::client::{get_value_input, parse_vec};
use ansi_term::Color;
use ansi_term::Style;
use star_realms::game::components::card::Card;

#[test]
pub fn test_input() {
    // oh wait, I can't test input
    // let n = get_value_input(|&i| 0 < i && i < 3);
    // println!("{}", n);
}

#[test]
fn test_parse_vec () {
    println!("printed vec: {:?}", vec![1, 2, 3]);
    let vec: Result<Vec<u32>, _> = parse_vec("1, 2,3,  4   , 5,6");
    println!("parsed vec: {:?}", vec);
}

#[test]
fn test_colored_text () {
    // https://rust-lang-nursery.github.io/rust-cookbook/cli/ansi_terminal.html
    println!("hello {}", Color::Red.paint("world"));
    println!("{}: {}", Color::Red.bold().paint("CLIENT ERROR"), Style::new().italic().paint("errr ererr"));
    // add colors <3 <3 <3
}