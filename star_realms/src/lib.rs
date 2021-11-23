#![allow(dead_code)]
pub mod game;
pub mod resources;
mod parse;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::mem;

    use yaml_rust::yaml::Yaml::Hash;
    use yaml_rust::YamlLoader;

    use crate::game::{GameState, PlayerArea, RelativePlayer};
    use crate::game::actions::{add_goods, draw_card, scrap_card, specially_place_next_acquired};
    use crate::game::card_library::CardLibrary;
    use crate::game::components::card::Card;
    use crate::game::components::card::details::{Action, Base, Exhaustibility, Play, Requirement, Sacrifice, CardSource, Actionable};
    use crate::game::components::faction::Faction;
    use crate::game::components::Goods;
    use crate::game::components::stack::Stack;
    use crate::game::util::Join;
    use crate::parse::{parse_card, parse_file, parse_goods};
    use crate::game::requirements::synergy;

    #[test]
    fn test_shuffle() {
        print_long_message("testing shuffle");
        for _ in 0..10 {
            let mut stack = Stack::new((1..5).collect());
            stack.shuffle();
            println!("{:?}", stack);
        }
    }

    #[test]
    fn test_yaml() {
        // yeah: don't test your libraries, but also idk how this works
        // so this will be left as an example for myself
        print_long_message("yaml test");
        let yaml = YamlLoader::load_from_str("\
card1:
  base: false
  name: bazinga
card2:
  base: false
  name: card2
        ");
        let yaml = &yaml.expect("not parsed correctly")[0];
        println!("{:?}", yaml);
        match yaml {
            Hash(b) => {
                for (k, v) in b {
                    println!("{:?} -> {:?}", k.as_str().unwrap(), v);
                }
            }
            _ => panic!("not a hash???")
        }
    }

    #[test]
    fn test_parse_goods () {
        assert_eq!(parse_goods("G6:3:0").unwrap(), Goods {
            combat: 6,
            authority: 3,
            trade: 0
        });
        assert_eq!(parse_goods("G12:0:4").unwrap(), Goods {
            combat: 12,
            authority: 0,
            trade: 4
        });
        assert_eq!(parse_goods("G144:225:124").unwrap(), Goods {
            combat: 144,
            authority: 225,
            trade: 124
        });
    }
    
    #[test]
    fn test_sizes () {
        println!("Size of String: {}", mem::size_of::<String>());
        println!("Size of CardLibrary: {}", mem::size_of::<CardLibrary>());
        println!("Size of GameState: {}", mem::size_of::<GameState>());
        println!("Size of Card: {}", mem::size_of::<Card>());
        println!("Size of &Card: {}", mem::size_of::<&Card>());
        println!("Size of PlayerArea: {}", mem::size_of::<PlayerArea>());
        println!("Size of HashSet<(String, String)>: {}", mem::size_of::<HashSet<(String, String)>>());
    }

    pub fn get_debug_cards() -> Vec<Card> {
        let card = Card {
            cost: 3,
            name: "Outland Station".to_string(),
            base: Some(Base::Base(4)),
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Fed);
                set
            },
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Action::Unit(
                        Join::choose(vec![
                            add_goods(Goods { trade: 1, authority: 0, combat: 0 }),
                            add_goods(Goods { trade: 0, authority: 3, combat: 0 })
                        ])
                    ),
                    exhaust: Exhaustibility::Once
                },
                Play {
                    cond: Some(
                        Join::Unit(
                            Requirement::Cost(
                                Sacrifice::ScrapThis
                            )
                        )
                    ),
                    actn: Action::Unit(Join::Unit(draw_card())),
                    exhaust: Exhaustibility::Once
                }
            ])
        };

        let card_2 = Card {
            cost: 7,
            name: "The Ark".to_string(),
            base: None,
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Mech);
                set
            },
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Action::Unit(
                        Join::Unit(add_goods(Goods::combat(5)))),
                    exhaust: Exhaustibility::Once
                },
                Play {
                    cond: Some(Join::Unit(
                        Requirement::Cost(Sacrifice::Scrap(
                            1,
                            Join::choose(vec![
                                CardSource::Discard(RelativePlayer::Current),
                                CardSource::Hand(RelativePlayer::Current)
                            ])
                        ))
                    )),
                    actn: Action::Unit(Join::Unit(draw_card())),
                    exhaust: Exhaustibility::UpTo(2)
                }
            ])
        };

        let card_3 = Card {
            name: "Trade Bot".to_string(),
            synergizes_with: {
                let mut set = HashSet::new();
                set.insert(Faction::Mech);
                set
            },
            base: None,
            cost: 1,
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Action::Unit(Join::all(vec![
                        add_goods(Goods::trade(1)),
                        scrap_card(
                            vec![CardSource::Discard(RelativePlayer::Current), CardSource::Hand(RelativePlayer::Current)]
                                .into_iter().collect())
                    ])),
                    exhaust: Exhaustibility::Once
                },
                Play {
                    cond: Some(Join::Unit(synergy(Faction::Mech))),
                    actn: Action::Unit(Join::Unit(add_goods(Goods::combat(2)))),
                    exhaust: Exhaustibility::Once
                }
            ])
        };

        let card_4 = Card {
            cost: 7,
            name: "Trade Envoy".to_string(),
            base: None,
            synergizes_with: vec![Faction::Fed].into_iter().collect(),
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Action::Unit(Join::all(vec![
                        add_goods(Goods { trade: 3, authority: 5, combat: 0 }),
                        draw_card()
                    ])),
                    exhaust: Exhaustibility::Once
                },
                Play {
                    cond: Some(Join::all(vec![
                        Requirement::Cost(Sacrifice::ScrapThis),
                        synergy(Faction::Fed)
                    ])),
                    actn: Action::Unit(Join::Unit(
                        specially_place_next_acquired(CardSource::Hand(RelativePlayer::Current))
                    )),
                    exhaust: Exhaustibility::Once
                }
            ])
        };

        let card_5 = Card {
            name: "Captured Outpost".to_string(),
            cost: 3,
            base: Some(Base::Outpost(3)),
            synergizes_with: vec![Faction::Star].into_iter().collect(),
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Action::Sequential(
                        Box::new(Join::Unit(
                            Action::Unit(Join::Unit(
                                draw_card()
                            ))
                        )),
                        Box::new(Join::Unit(
                            Action::Unit(Join::Unit(
                                // discard_card()
                                todo!()
                            ))
                        ))
                    ),
                    exhaust: Exhaustibility::Once
                }
            ])
        };

        let card_6 = Card {
            cost: 1,
            synergizes_with: vec![Faction::Fed].into_iter().collect(),
            name: "Cargo Rocket".to_string(),
            base: None,
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Action::Unit(Join::Unit(add_goods(Goods {
                        authority: 3,
                        combat: 2,
                        trade: 1
                    }))),
                    exhaust: Exhaustibility::Once
                },
                Play {
                    cond: Some(Join::all(vec![
                        synergy(Faction::Fed),
                        Requirement::Cost(Sacrifice::ScrapThis)
                    ])),
                    actn: Action::Unit(Join::Unit(
                        Actionable::no_args(|game, _| {
                            // add a trigger to the game
                            // "Put the next ship you acquire this turn on top of your deck"
                            todo!()
                        })
                    )),
                    exhaust: Exhaustibility::Once
                }
            ])
        };
        vec![
            card,
            card_2,
            card_3,
            card_4,
            card_5,
            card_6
        ]
    }

    #[test]
    fn join_playground() {
        let joined = Join::Unit(5);
        let v = vec![1, 2, 3, 4, 5];
        let joined_2 = Join::all(v);
        println!("{:?}", if let Join::All(xs) = joined_2 {
            xs
        } else {
            vec![]
        })
    }

    fn print_long_message(msg: &str) {
        println!();
        println!("==================={}===================", msg);
    }
}
