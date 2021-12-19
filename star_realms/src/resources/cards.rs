use crate::game::components::card::Card;
use crate::game::components::card::details::{Base, Play, Action, Exhaustibility, Actionable, Requirement, Sacrifice, CardSource};
use crate::game::components::faction::Faction;
use crate::game::util::{Join, Named};
use std::collections::HashSet;
use crate::game::actions::{add_goods, draw_card, specially_place_next_acquired, scrap_card};
use crate::game::components::Goods;
use crate::game::requirements::synergy;
use crate::game::RelativePlayer;

pub fn get_misc_cards() -> Vec<Card> {
    vec![
        Card {
            cost: 0,
            base: None,
            synergizes_with: HashSet::new(),
            name: "scout".to_string(),
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Named::of("Gain 1 trade", Action::Unit(Join::Unit(add_goods(Goods::trade(1))))),
                    exhaust: Exhaustibility::Once
                }
            ])
        },
        Card {
            cost: 0,
            base: None,
            synergizes_with: HashSet::new(),
            name: "viper".to_string(),
            content: Some(vec![
                Play {
                    cond: None,
                    actn: Named::of("Gain 1 combat", Action::Unit(Join::Unit(add_goods(Goods::combat(1))))),
                    exhaust: Exhaustibility::Once
                }
            ])
        }
    ]
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
                actn: Named::of("Add 1 trade or 3 authority", Action::Unit(
                    Join::disjoint(vec![
                        add_goods(Goods { trade: 1, authority: 0, combat: 0 }),
                        add_goods(Goods { trade: 0, authority: 3, combat: 0 })
                    ])
                )),
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
                actn: Named::of("Draw card", Action::Unit(Join::Unit(draw_card()))),
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
                actn: Named::of("Add 5 combat", Action::Unit(
                    Join::Unit(add_goods(Goods::combat(5))))),
                exhaust: Exhaustibility::Once
            },
            Play {
                cond: Some(Join::Unit(
                    Requirement::Cost(Sacrifice::Scrap(
                        1,
                        Join::disjoint(vec![
                            CardSource::Discard(RelativePlayer::Current),
                            CardSource::Hand(RelativePlayer::Current)
                        ])
                    ))
                )),
                actn: Named::of("Draw card", Action::Unit(Join::Unit(draw_card()))),
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
                actn: Named::of("Gain 1 trade and scrap a card in your hand or discard pile", Action::Unit(Join::union(vec![
                    add_goods(Goods::trade(1)),
                    scrap_card(
                        vec![CardSource::Discard(RelativePlayer::Current), CardSource::Hand(RelativePlayer::Current)]
                            .into_iter().collect())
                ]))),
                exhaust: Exhaustibility::Once
            },
            Play {
                cond: Some(Join::Unit(synergy(Faction::Mech))),
                actn: Named::of("Gain 2 combat", Action::Unit(Join::Unit(add_goods(Goods::combat(2))))),
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
                actn: Named::of("Gain 3 trade and 5 authority, and draw a card", Action::Unit(Join::union(vec![
                    add_goods(Goods { trade: 3, authority: 5, combat: 0 }),
                    draw_card()
                ]))),
                exhaust: Exhaustibility::Once
            },
            Play {
                cond: Some(Join::union(vec![
                    Requirement::Cost(Sacrifice::ScrapThis),
                    synergy(Faction::Fed)
                ])),
                actn: Named::of("Put the next card you acquire in your hand", Action::Unit(Join::Unit(
                    specially_place_next_acquired(CardSource::Hand(RelativePlayer::Current))
                ))),
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
                actn: Named::of("Draw a card, then discard a card", Action::Sequential(
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
                )),
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
                actn: Named::of("Gain 1 trade, 2 combat, and 3 trade", Action::Unit(Join::Unit(add_goods(Goods {
                    authority: 3,
                    combat: 2,
                    trade: 1
                })))),
                exhaust: Exhaustibility::Once
            },
            Play {
                cond: Some(Join::union(vec![
                    synergy(Faction::Fed),
                    Requirement::Cost(Sacrifice::ScrapThis)
                ])),
                actn: Named::of("Put the next ship you acquire this turn on top of your deck", Action::Unit(Join::Unit(
                    specially_place_next_acquired(CardSource::Deck(RelativePlayer::Current)))
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