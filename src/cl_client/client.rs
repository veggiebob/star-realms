use std::collections::{HashMap, HashSet};
use std::io;
use std::str::FromStr;

use ansi_term::Color;
use star_realms::game::{AbstractPlayerAction, Feedback, GameState, RelativePlayer, UserActionIntent, UserActionSupplier};
use star_realms::game::actions::client_comms::{Client, ClientQuery, ClientActionOptionResponse};

pub struct ClientPTUI {
    pub name: String
}

impl Client for ClientPTUI {
    fn resolve_action_query(query: ClientQuery) -> ClientActionOptionResponse {
        todo!()
    }
}

fn print_options<T: ToString>(options: &Vec<T>) {
    for (index, element) in options.iter().enumerate() {
        println!(" {} - {}", Color::Blue.paint(index.to_string()), element.to_string());
    }
}

fn input() -> String {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    (&s[0..s.len()-1]).to_string()
}

/// Ensure that standard input results in a value that is satisfied by `valid`,
/// using parsed strings and io interaction
pub fn get_value_input<T: FromStr, U: FnMut(&T) -> bool>(valid: U) -> T {
    ensure(
        input,
        |s| s.parse(),
        valid,
        |_| println!("invalid input"),
        |s| println!("invalid input of '{:?}'", s)
    )
}

/// A more generic `get_value_input` that uses `gen` to get string values,
/// parses them using the `parse` function,
/// returns the one that meets `valid`,
/// and calls `fail` for failed inputs
pub fn ensure<T, G, P, F, U, E, H>(
    mut gen: G,
    mut parse: P,
    mut valid: U,
    mut failed_input: F,
    mut failed_parse: H) -> T
    where G: FnMut() -> String,
          U: FnMut(&T) -> bool,
          F: FnMut(&T),
          P: FnMut(&String) -> Result<T, E>,
          H: FnMut(&String) {
    loop {
        let s = gen();
        let value = parse(&s);
        if let Ok(v) = value {
            if valid(&v) {
                return v;
            }
            failed_input(&v);
        } else {
            failed_parse(&s);
        }
    }
}


/// because parsing doesn't exist for Vec<T: FromStr>???

pub fn parse_vec <T: FromStr> (input: &str) -> Result<Vec<T>, ()> {
    let split: Vec<_> = input.split(',').collect();
    let mut out = vec![];
    for s in split {
        match s.trim().parse() {
            Ok(s) => out.push(s),
            Err(_) => return Err(())
        }
    }
    Ok(out)
}

pub struct ParsedVec<T>(Vec<T>);
impl<T> ParsedVec<T> {
    pub fn vec(self) -> Vec<T> {
        self.0
    }
}
impl<T: FromStr> FromStr for ParsedVec<T> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_vec(s) {
            Ok(v) => Ok(ParsedVec(v)),
            Err(_) => Err(())
        }
    }
}

