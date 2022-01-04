use crate::game::components::card::details::{ExhaustionLevel, Play, Exhaustibility};

pub struct ActivePlay {
    // would like to borrow here, but that's a lot of refactoring that I don't want to do
    // considering that Play is Clone, it's not much a problem to do this, right?
    pub play: Play,
    pub times_used: ExhaustionLevel
}

impl ActivePlay {
    pub fn new(play: &Play) -> ActivePlay {
        ActivePlay {
            play: play.clone(),
            times_used: 0
        }
    }
    pub fn not_exhausted(&self) -> bool {
        match self.play.exhaust {
            Exhaustibility::Once => self.times_used == 0,
            Exhaustibility::UpTo(x) => self.times_used < x,
            // exactly will probably be used in some other way
            Exhaustibility::Exactly(x) => self.times_used < x
        }
    }

    /// Returns None if it's exhausted. Otherwise, returns the number of exhaustion levels left.
    pub fn exhaustions_left(&self) -> Option<ExhaustionLevel> {
        if self.not_exhausted() {
            Some(
                match self.play.exhaust {
                    Exhaustibility::Once => 1,
                    Exhaustibility::UpTo(n) => n - self.times_used,
                    Exhaustibility::Exactly(n) => n - self.times_used
                }
            )
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Trigger();