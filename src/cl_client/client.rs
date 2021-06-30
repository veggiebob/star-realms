use star_realms::game::{UserActionSupplier, Feedback, GameState, UserActionIntent, RelativePlayer};
use std::collections::HashSet;
use star_realms::game::effects::{ConfigSupplier, Config, ActionConfigMethod};
use std::io;
use std::io::Error;
use std::str::FromStr;

pub struct Client {
    pub name: String
}

impl UserActionSupplier for Client {
    fn select_effect(&self, game: &GameState) -> UserActionIntent<(u32, (String, String))> {
        todo!("Client::select_effect")
    }

    fn select_card(&self, game: &GameState, from_who: &RelativePlayer) -> u32 {
        todo!("Client::select_card")
    }

    fn select_cards(&self, game: &GameState, from_who: &RelativePlayer) -> HashSet<u32> {
        todo!("Client::select_cards")
    }

    fn on_feedback(&self, feedback: Feedback) {
        todo!("Client::on_feedback")
    }
}

fn input() -> String {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s
}

fn get_value_input<T: FromStr, U: FnMut(&T) -> bool>(mut valid: U) -> T {
    loop {
        if let Ok(i) = input().parse() {
            if valid(&i) {
                return i;
            }
        }
        println!("invalid input.");
    }
}

// fn ensure<T, G: FnMut() -> T, U: FnMut(&T) -> bool, F: FnMut(&T)>(mut gen: G, mut valid: U, &mut fail: Option<F>) -> T {
//     let does_fail = fail.is_some();
//     loop {
//         let value = gen();
//         if valid(&value) {
//             return value;
//         }
//         if does_fail {
//             (fail.unwrap())(&value);
//         }
//     }
// }

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