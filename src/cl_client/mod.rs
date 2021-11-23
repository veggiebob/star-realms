pub mod client;

#[cfg(test)]
pub mod test;
pub mod main {
    extern crate star_realms;

    use ansi_term::Color;
    use crate::cl_client::client::Client;
    use self::star_realms::game::GameState;
    use self::star_realms::game::card_library::CardLibrary;
    use self::star_realms::resources::cards::{get_misc_cards, get_debug_cards};
    use std::rc::Rc;

    pub fn main () {
        println!("RUNNING IN STABLE (???) MODE (???)");
        println!("cl_client::main::main: Game has ended!");
    }
    pub fn debug () {
        println!("{}", Color::Red.paint("***** ***** RUNNING IN DEBUG MODE ***** *****"));
        let cards = get_debug_cards();
        let misc_cards = get_misc_cards();
        let mut cl = Rc::new(CardLibrary::new(cards, misc_cards).unwrap());
        let mut game = GameState::new(cl);
        let client = Client {
            name: "debug user!".to_string()
        };
        // loop {
        //     game.advance(&client);
        // }
        println!("cl_client::main::debug: Game has ended!");
    }
}