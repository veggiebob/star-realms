use crate::game::{effects, GameState};

pub fn get_condition(name: String) -> Option<ConditionFunc> {
    match name.as_str() {
        _ if effects::is_free_cond(&name) => Some(Box::new(|_, _| true)),
        _ if effects::is_trash_cond(&name) => Some(Box::new(
            |game, id| {
                game.get_current_player().hand_id.get(id)
                    .expect("trash condition: bad id supplied")
                    .1.scrapped
            }
        )),
        // example: "syn t" for synergy with Trade Federation
        _ if name.starts_with("syn") => Some(Box::new({
                let n = name.clone();
                move |game, id| match &(n.as_str()[n.len()-1..].parse()) {
                    Ok(p) => game.get_current_player()
                        .get_card_in_hand(id)
                        .expect("synergy condition: bad id supplied")
                        .0.synergizes_with.contains(p),
                    Err(e) => panic!("'{}' is not a valid condition! {}", &n, e)
                }
            })
        ),
        _ => None
    }
}

/// FnMut(game, hand_id /* of card */) -> bool
pub type ConditionFunc = Box<dyn FnMut(&GameState, &u32) -> bool>;
