pub mod client;

#[cfg(test)]
pub mod test;
pub mod main {
    extern crate star_realms;

    use self::star_realms::game::{GameState, Feedback};
    use crate::cl_client::client::Client;

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
                Ok(msg) => println!("log: {}", msg),
                Err(e) => {
                    println!("Internal unrecoverable error.");
                    println!("{}", e);
                    break;
                }
            }
        }
        println!("cl_client::main::main: Game has ended!");
    }
}