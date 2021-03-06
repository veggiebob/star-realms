pub mod client;

#[cfg(test)]
pub mod test;
pub mod main {
    extern crate star_realms;

    use self::star_realms::game::{GameState};
    use crate::cl_client::client::Client;
    use ansi_term::Color;

    pub fn main () {
        let mut game = match GameState::from_config("star_realms/config") {
            Ok(g) => g,
            Err(e) => panic!("Could not create game: {}", e)
        };
        let client = Client {
            name: "user".to_string()
        };
        println!("cl_client::main::main: Game is starting!");
        loop {
            let result = game.advance(&client);
            match result {
                Ok(msg) => println!("{}", Color::Yellow.paint(msg)),
                Err(e) => {
                    println!("{}", Color::Red.paint("Internal unrecoverable error."));
                    println!("{}", Color::Red.paint(e));
                    break;
                }
            }
        }
        println!("cl_client::main::main: Game has ended!");
    }
    pub fn debug () {
        println!("{}", Color::Red.paint("***** ***** RUNNING IN DEBUG MODE ***** *****"));
        let mut game = match GameState::from_config("star_realms/config") {
            Ok(g) => g,
            Err(e) => panic!("Could not create game: {}", e)
        };

        let client = Client {
            name: "dev".to_string()
        };
        println!("cl_client::main::debug: Game is starting!");
        // game.get_current_player_mut().end_turn();
        // let explorer = (*game.card_library.get_explorer().unwrap()).clone();
        // game.get_current_player_mut().give_card_to_hand(explorer);
        let stealth_needle = (*game.card_library.get_card_by_name("stealth_needle").unwrap()).clone();
        game.get_current_player_mut().give_card_to_hand(stealth_needle);
        loop {
            let result = game.advance(&client);
            match result {
                Ok(msg) => println!("log: {}", Color::Yellow.paint(msg)),
                Err(e) => {
                    println!("Internal unrecoverable error.");
                    println!("{}", e);
                    break;
                }
            }
        }
        println!("cl_client::main::debug: Game has ended!");
    }
}