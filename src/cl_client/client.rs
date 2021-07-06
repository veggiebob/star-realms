use star_realms::game::{UserActionSupplier, Feedback, GameState, UserActionIntent, RelativePlayer, PlayerArea};
use std::collections::{HashSet, HashMap};
use star_realms::game::effects::{ConfigSupplier, Config, ActionConfigMethod, get_condition};
use std::io;
use std::io::Error;
use std::str::FromStr;

pub struct Client {
    pub name: String
}

impl UserActionSupplier for Client {
    fn select_effect(&self, game: &GameState) -> UserActionIntent<(u32, (String, String))> {
        println!("Select an action:");
        let mut index = 1;
        let cp = game.get_current_player();
        let ids = cp.get_all_hand_card_ids();
        let mut enumerated = HashMap::new();
        let mut card_index_map = HashMap::new();

        println!(" 0: Skip effects");
        for (id, (card, card_status)) in ids.iter().map(|id| (id, cp.get_card_in_hand(id).unwrap())) {
            let unused_effects:Vec<_> = card.effects
                .iter()
                .filter(|e| !card_status.effects_used.contains(e))
                .collect();
            if !unused_effects.is_empty() {
                println!("{}:", &card.name);
                // println!("All effects: {:?}", card.effects);
                for effect in unused_effects {
                    println!(" {}: {:?}", index, effect);
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
                return UserActionIntent::Finish;
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
        println!("Would you like to look at the trade row? Enter 'y' for yes.");
        if input().is_empty() {
            return UserActionIntent::Finish
        }
        println!("Select a card by number, less than {}", game.trade_row.len());
        if game.explorers > 0 {
            println!(" 0 - explorer ({} left)", game.explorers);
        }
        let index = 1;
        for id in game.trade_row.elements.iter() {
            let card = game.card_library.as_card(id);
            println!(" {} - {} ({})", index, card.name, card.cost);
        }
        UserActionIntent::Continue(get_value_input(|&i| {
            i <= game.trade_row.len() as u32 && (i > 0 || game.explorers > 0)
        }))
    }
    fn on_feedback(&self, feedback: Feedback) {
        match feedback {
            Feedback::Invalid(msg) => println!("Invalid action! {}", msg),
            Feedback::Info(msg) => {
                println!("Just letting you know...");
                println!("{}", msg);
            }
        }
    }
}

fn summarize_hand(player: &PlayerArea) -> String {
    let mut str = String::new();
    for id in player.get_all_hand_card_ids().iter() {
        str += format!(" - {}: {}\n", id, player.get_card_in_hand(id).unwrap().0.name.clone()).as_str();
    }
    str
}

fn input() -> String {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    (&s[0..s.len()-1]).to_string()
}

/// ensure that standard input results in a value that is satisfied by `valid`,
/// using parsed strings and io interaction
pub fn get_value_input<T: FromStr, U: FnMut(&T) -> bool>(mut valid: U) -> T {
    loop {
        let s = input();
        if let Ok(i) = s.parse() {
            if valid(&i) {
                return i;
            }
        }
        println!("invalid input of {:?}", s);
    }
}

/// A more generic `get_value_input` that uses `gen` to get values,
/// returning the one that meets `valid`
/// and calling `fail` on failed inputs
pub fn ensure<T, G: FnMut() -> T, U: FnMut(&T) -> bool, F: FnMut(&T)>(mut gen: G, mut valid: U, mut fail: F) -> T {
    loop {
        let value = gen();
        if valid(&value) {
            return value;
        }
        fail(&value)
    }
}

impl ConfigSupplier for Client {
    fn get_config(&self, game: &GameState, config: &Config) -> u32 {
        match &config.config_method {
            ActionConfigMethod::Range(a, b) => {
                println!("enter a number between {} and {}", a, b);
                get_value_input(|n| a <= n && n <= b)
            }
            ActionConfigMethod::Set(set) => {
                println!("enter one of the following: {:?}", set);
                get_value_input(|n| set.contains(n))
            },
            ActionConfigMethod::PickHandCard(by, from) => {
                if let RelativePlayer::Current = by {
                    println!("current player picks this card");
                } else {
                    println!("opponent picks this card");
                }
                let hand = game.resolve_relative_player(from);
                let set = hand.get_all_hand_card_ids();
                println!("pick a card from the {} player's hand:", from.to_string());
                for s in set.iter() {
                    println!("{}: {}", s, hand.get_card_in_hand(s).unwrap().0.name);
                }
                get_value_input(|n| set.contains(n))
            },
            ActionConfigMethod::PickHandCards(num, by, from) => {
                todo!()
            },
            ActionConfigMethod::PickTradeRowCards(num, by) => todo!(),
        }
    }
}

