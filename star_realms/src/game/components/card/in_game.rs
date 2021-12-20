use crate::game::components::card::details::{ExhaustionLevel, Play, Exhaustibility};

pub struct ActivePlay<'a> {
    pub play: &'a Play,
    pub times_used: ExhaustionLevel
}

impl ActivePlay<'_> {
    pub fn new<'a>(play: &'a Play) -> ActivePlay<'a> {
        ActivePlay {
            play,
            times_used: 0
        }
    }
    pub fn not_exhausted(&self) -> bool {
        match self.play.exhaust {
            Exhaustibility::Once => self.times_used == 0,
            Exhaustibility::UpTo(x) => self.times_used < x,
            Exhaustibility::Exactly(x) => self.times_used < x
        }
    }
}

#[derive(Debug)]
pub struct Trigger();