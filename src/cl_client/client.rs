use star_realms::game::{UserActionSupplier, Feedback, GameState, UserActionIntent, RelativePlayer, AbstractPlayerAction};
use std::collections::{HashSet, HashMap};
use star_realms::game::effects::{ConfigSupplier, Config, ActionConfigMethod, get_condition};
use std::io;
use std::str::FromStr;
use ansi_term::Color;
use star_realms::game::components::card::CardStatus;

pub struct Client {
    pub name: String
}
impl Client {
    fn pick_hand_card(&self, game: &GameState, by: &RelativePlayer, from: &RelativePlayer) -> u32 {
        if by == from {
            println!("{:?}, pick one of your cards", by);
        } else {
            println!("{:?}, pick a card from {:?}", by, from);
        }
        let player = game.resolve_relative_player(from);
        for i in player.get_all_hand_card_ids().iter() {
            let (card, _) = player.get_card_in_hand(i).unwrap();
            println!(" {} - {}", i, card.name);
        }
        get_value_input(|i| player.get_all_hand_card_ids().contains(i))
    }
    fn pick_hand_cards(&self, game: &GameState, by: &RelativePlayer, from: &RelativePlayer, num: &u32) -> u32 {
        if by == from {
            println!("{:?}, pick {} of your cards", by, num);
        } else {
            println!("{:?}, pick {} cards from {:?}", by, num, from);
        }
        let player = game.resolve_relative_player(from);
        println!("enter comma separated values for {} of the cards you want:", num);
        let ids = player.get_all_hand_card_ids();
        let ids:Vec<_> = ids.iter().collect();
        let mut sorted_ids = ids.clone();
        sorted_ids.sort();
        for (index, &id) in sorted_ids.iter().enumerate() {
            println!(" {} - {:?}", index, player.get_card_in_hand(id).unwrap());
        }
        let choices: ParsedVec<u32> = get_value_input(|vs: &ParsedVec<u32>| {
            // first, check for duplicates
            let vs = &vs.0;
            let mut set = HashSet::new();
            for i in vs.iter() {
                if set.contains(i) {
                    return false;
                }
                set.insert(i);
            }
            // then make sure that all of them are valid inputs
            vs.iter().all(|&n| n < sorted_ids.len() as u32)
        });
        let choices = choices.0;
        let mut n = 0;
        for &c in choices.iter() {
            n |= 1<<c;
        }
        println!("DEBUG Client::pick_hand_cards: cards: {:?} converted to {}", choices, n);
        n
    }
}
impl UserActionSupplier for Client {
    fn choose_abstract_action(&self, _game: &GameState) -> AbstractPlayerAction {
        println!("Select an action:");
        let options = vec![
            "Use effects on cards",
            "View trade row",
            "Trash a card",
            "End Turn"
            ];
        print_options(&options);
        match get_value_input(|&i: &u8| i < 4) {
            0 => AbstractPlayerAction::CardEffects,
            1 => AbstractPlayerAction::TradeRow,
            2 => AbstractPlayerAction::TrashCard,
            3 | _ => AbstractPlayerAction::EndTurn,
        }
    }
    fn select_effect(&self, game: &GameState) -> UserActionIntent<(u32, (String, String))> {
        println!("Select an action:");
        let mut index = 1;
        let cp = game.get_current_player();
        let ids = cp.get_all_hand_card_ids();
        let mut enumerated = HashMap::new();
        let mut card_index_map = HashMap::new();

        println!(" {}: Skip effects", Color::Blue.paint("0"));
        for (id, (card, card_status)) in ids.iter().map(|id| (id, cp.get_card_in_hand(id).unwrap())) {
            let unused_effects = card_status.unused_effects(card);
            if !unused_effects.is_empty() {
                println!("{}:", &card.name);
                // println!("All effects: {:?}", card.effects);
                for effect in unused_effects {
                    if CardStatus::is_free(&effect.1) {
                        match card_status.get_good(&effect.1) {
                            Some(g) => println!(
                                " {} - {}",
                                Color::Blue.paint(index.to_string()),
                                g
                            ),
                            None => println!(
                                " {} - {}",
                                Color::Blue.paint(index.to_string()),
                                &effect.1
                            )
                        }
                    } else {
                        match card_status.get_good(&effect.1) {
                            Some(g) => println!(" {} - {} => {}", Color::Blue.paint(index.to_string()),
                                                effect.0, g
                            ),
                            None => println!(" {} - {} => {}", Color::Blue.paint(index.to_string()),
                                             effect.0, effect.1
                            )
                        }
                    }
                    enumerated.insert(index, effect.clone());
                    card_index_map.insert(index, id);
                    index += 1;
                }
            }
        }
        let (eff, card_id) = loop {
            println!("input a positive number less than {}", index);
            let index: &u32 = &get_value_input(|i| *i < index);
            if *index == 0 {
                return UserActionIntent::Cancel;
            }
            let id = card_index_map.get(index).unwrap().clone();
            let e = enumerated.get(index).unwrap().clone();
            if !(get_condition(e.0.clone()).unwrap())(game, id) {
                println!("The condition {} was not met. Please try a different effect", &e.0);
                continue;
            } else {
                break (e, id);
            }
        };
        UserActionIntent::Continue((*card_id, eff))
    }
    fn select_trade_row_card(&self, game: &GameState) -> UserActionIntent<u32> {
        println!("Select a card by number, less than {}", game.trade_row.len());
        if game.explorers > 0 {
            println!(" {} - explorer ({} left)", Color::Blue.paint("0"), game.explorers);
        }
        let index = 1;
        for id in game.trade_row.elements.iter() {
            let card = game.card_library.as_card(id);
            println!(" {} - {} ({})", Color::Blue.paint(index.to_string()), card.name, Color::Yellow.paint(card.cost.to_string()));
        }
        UserActionIntent::Continue(get_value_input(|&i| {
            i <= game.trade_row.len() as u32 && (i > 0 || game.explorers > 0)
        }))
    }
    fn on_feedback(&self, feedback: Feedback) {
        match feedback {
            Feedback::Invalid(msg) => println!("{} {}", Color::Red.paint("Invalid action!"), Color::Red.paint(msg)),
            Feedback::Info(msg) => {
                println!("{}", Color::Yellow.paint("Just letting you know..."));
                println!("{}", Color::Yellow.paint(msg));
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

impl ConfigSupplier for Client {
    fn get_config(&self, game: &GameState, config: &Config) -> u32 {
        let v = match &config.config_method {
            ActionConfigMethod::Range(a, b) => {
                println!("enter a number between {} and {}", a, b);
                get_value_input(|n| a <= n && n <= b)
            }
            ActionConfigMethod::Set(set) => {
                println!("enter one of the following: {:?}", set);
                get_value_input(|n| set.contains(n))
            },
            ActionConfigMethod::PickHandCard(by, from) => {
                self.pick_hand_card(game, by, from)
            },
            ActionConfigMethod::PickHandCards(num, by, from) => {
                self.pick_hand_cards(game, by, from, num)
            },
            ActionConfigMethod::PickTradeRowCards(num, by) => {
                println!("{:?}, pick {} of the trade row cards", by, num);
                for (index, id) in game.trade_row.iter().enumerate() {
                    println!(" {} - {}", Color::Blue.paint(index.to_string()), game.card_library.as_card(id).name);
                }
                let mut sorted_trade_row = game.trade_row.elements.clone();
                sorted_trade_row.sort();
                // have to use wrapper type which implements FromStr
                let input: ParsedVec<u32> = get_value_input(|vs: &ParsedVec<u32>| {
                    let vs = &vs.0;
                    // ensure that there are no duplicates
                    let set: HashSet<_> = vs.iter().collect();
                    if set.len() < vs.len() {
                        return false;
                    }
                    // then make sure that all of them are valid inputs
                    vs.iter().all(|&n| n < sorted_trade_row.len() as u32)
                });
                let input = input.0;
                let mut out: u32 = 0;
                for &i in input.iter() {
                    out |= u32::pow(2, i);
                }
                println!("DEBUG Client::get_config: trade row cards: {:?} converted to {}",
                         input, out);
                out
            },
        };
        // my IDE can't handle this apparently lmao
        println!("{}\nAre you sure? (y/n)", (config.describe)(v).as_str());
        match input().as_str() {
            "y" => v,
            "n" | _ => self.get_config(game, config)
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

