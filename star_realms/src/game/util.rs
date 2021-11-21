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

#[derive(Debug, Clone)]
pub enum Join<T: Clone> {
    Unit(T),
    All(Vec<Box<Join<T>>>),
    Choose(Vec<Box<Join<T>>>)
}

impl<T: Clone> Join<T> {
    pub fn unit(item: T) -> Join<T> {
        Join::Unit(item)
    }
    pub fn all(items: Vec<T>) -> Join<T> {
        Join::All(items.into_iter().map(Join::Unit).map(Box::new).collect())
    }
    pub fn choose(items: Vec<T>) -> Join<T> {
        Join::Choose(items.into_iter().map(Join::Unit).map(Box::new).collect())
    }
}