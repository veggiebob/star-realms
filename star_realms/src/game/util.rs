use std::fmt::Display;

pub enum Failure<T> {
    Fail(T),
    Succeed
}

impl<T: Display> Failure<T> {
    pub fn check(&self) {
        if let Failure::Fail(message) = self {
            panic!("Failure was unwrapped! {}", message);
        }
    }
}

#[derive(Debug)]
pub enum Join<T> {
    Unit(T),
    All(Vec<Box<Join<T>>>),
    Choose(Vec<Box<Join<T>>>)
}

impl<T> Join<T> {
    pub fn all<I: Iterator<Item=T>>(items: I) -> Join<T> {
        Join::All(items.map(Join::Unit).map(Box::new).collect())
    }
    pub fn choose<I: Iterator<Item=T>>(items: I) -> Join<T> {
        Join::Choose(items.map(Join::Unit).map(Box::new).collect())
    }
}