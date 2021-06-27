use star_realms::game::{UserActionSupplier, Feedback, GameState, UserActionIntent, RelativePlayer};
use std::collections::HashSet;
use star_realms::game::effects::{ConfigSupplier, Config};

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

impl ConfigSupplier for Client {
    fn get_config(&self, game: &GameState, config: &Config) -> u32 {
        todo!("Client::get_config")
    }
}