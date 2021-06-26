use std::fmt::Display;

pub enum Failure<T> {
    Failure(T),
    Success
}

impl<T: Display> Failure<T> {
    pub fn check(&self) {
        if let Failure::Failure(message) = self {
            panic!(format!("Failure was unwrapped! {}", message));
        }
    }
}
