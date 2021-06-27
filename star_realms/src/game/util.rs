use std::fmt::Display;

pub enum Failure<T> {
    Fail(T),
    Succeed
}

impl<T: Display> Failure<T> {
    pub fn check(&self) {
        if let Failure::Fail(message) = self {
            panic!(format!("Failure was unwrapped! {}", message));
        }
    }
}
