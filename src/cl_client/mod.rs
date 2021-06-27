pub mod client;

pub mod main {
    extern crate star_realms;

    use self::star_realms::game::GameState;
    use crate::cl_client::client::Client;

    pub fn main () {
        let mut game = match GameState::from_config("star_realms/config") {
            Ok(g) => g,
            Err(e) => panic!("Could not create game: {}", e)
        };
        let client = Client {
            name: "user".to_string()
        };
        let result = game.advance(&client);
    }
}