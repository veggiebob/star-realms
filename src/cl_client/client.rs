use std::collections::{HashMap, HashSet};
use std::io;
use std::str::FromStr;

use ansi_term::Color;
use star_realms::game::{AbstractPlayerAction, Feedback, GameState, RelativePlayer, UserActionIntent, Player};
use star_realms::game::actions::client_comms::{Client, ClientQuery, ClientActionOptionResponse, TextStyle};
use star_realms::game::components::card::details::CardSource;
use std::fmt::Display;

pub struct ClientPTUI {
    pub name: String
}

impl ClientPTUI {
    fn p1<T: Display>(msg: T) -> String {
        format!("{}", Color::Blue.paint(format!("{}", msg)))
    }
    fn p2<T: Display>(msg: T) -> String {
        format!("{}", Color::Purple.paint(format!("{}", msg)))
    }
    fn p_colored<T: Display>(player: &Player, msg: T) -> String {
        match player {
            Player::Player1 => format!("{}", ClientPTUI::p1(msg)),
            Player::Player2 => format!("{}", ClientPTUI::p2(msg))
        }
    }
}

impl Client for ClientPTUI {
    fn resolve_action_query(&mut self, query: ClientQuery) -> ClientActionOptionResponse {
        println!("received a query, for user {:?}", query.performer);
        println!("The query is {:?}", query.action_query);
        println!("Since this is stubbed, returning a bad response");
        ClientActionOptionResponse::CardSelection(CardSource::Deck(RelativePlayer::Current), 0) // idk this is not accurate
    }

    fn alert<'a, T: Eq>(&self, message: &HashMap<Player, &str>, interrupt: &HashMap<Player, Option<Vec<(&str, &'a T)>>>, style: TextStyle) -> Option<&'a T> {
        println!("alert received.");
        for (player, msg) in message.iter() {
            println!("{:?}, {}", player, ClientPTUI::p_colored(player, msg));
        }
        loop {
            let mut responses = HashMap::new();
            for (player, interrupt) in interrupt.iter() {
                match interrupt {
                    Some(options) => {
                        println!("{:?}, {}", player,
                                 ClientPTUI::p_colored(player, "Please select one of the following options using a number."));
                        let mut idx: u32 = 0;
                        for (desc, _) in options.iter() {
                            println!("{}. {}", idx, desc);
                            idx += 1;
                        }
                        let response: u32 = get_value_input(|u| *u < idx);
                        let (_, response) = options.get(response as usize).unwrap();
                        responses.insert(player, response);
                    },
                    None => {
                        println!("{}", ClientPTUI::p_colored(player, "Ok?"));
                    }
                }
            }
            let r1 = responses.get(&Player::Player1);
            let r2 = responses.get(&Player::Player2);
            if r1.is_some() && r2.is_some() {
                if r1.unwrap() != r2.unwrap() {
                    println!("{}",
                             Color::Red.paint("Player responses must match!"));
                    continue;
                } else {
                    return Some(r1.unwrap())
                }
            } else if r1.is_none() && r2.is_none() {
                return None
            } else {
                return r1.or_else(|| r2).map(|&&x| x);
            }
        }
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

