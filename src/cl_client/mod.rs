pub mod client;

#[cfg(test)]
pub mod test;
pub mod main {
    extern crate star_realms;

    use ansi_term::Color;
    use crate::cl_client::client::ClientPTUI;
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
        let cl = Rc::new(match CardLibrary::new(cards, misc_cards) {
            Ok(cl) => cl,
            Err(e) => panic!("Unable to create Card Library. {}", Color::Red.paint(e))
        });
        let mut game = GameState::new(cl, 80);
        let mut client = ClientPTUI {
            name: "debug user!".to_string()
        };
        loop {
            // this could be the idle state?
            // it's kinda scuffed, but this game
            // really isn't meant to be played as a PTUI
            // todo: let the player see some cards!
            game.advance(&mut client);
        }
        println!("cl_client::main::debug: Game has ended!");
    }
}