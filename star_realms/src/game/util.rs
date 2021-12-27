use std::fmt::{Display, Formatter, Debug};
use crate::game::util::Failure::{Fail, Succeed};

pub enum Failure<T> {
    Fail(T),
    Succeed
}

#[derive(Clone)]
pub struct Named<T> {
    pub name: String,
    pub item: T,
}

impl<T: Display> Failure<T> {
    pub fn check(&self) {
        if let Failure::Fail(message) = self {
            panic!("Failure was unwrapped! {}", message);
        }
    }
}
impl<T> Failure<T> {
    pub fn as_result(self) -> Result<(), T> {
        match self {
            Fail(x) => Err(x),
            Succeed => Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum Join<T: Clone> {
    Unit(T),
    Union(Vec<Box<Join<T>>>),
    Disjoint(Vec<Box<Join<T>>>)
}

impl<T: Clone> Join<T> {
    fn from_vec(vec: Vec<T>, union: bool) -> Join<T> {
        if vec.len() == 0 {
            panic!("you idiot. Don't try to convert an empty vec into Join<T>");
        } else if vec.len() == 1 {
            let mut v = vec;
            Join::Unit(v.remove(0))
        } else {
            let v = vec.into_iter().map(Join::Unit).map(Box::new).collect();
            if union {
                Join::Union(v)
            } else {
                Join::Disjoint(v)
            }
        }
    }
}

impl<T: Clone> From<T> for Join<T> {
    fn from(x: T) -> Self {
        Join::Unit(x)
    }
}

impl<T: Display + Clone> Display for Join<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Join::Unit(x) => x.fmt(f),
            Join::Union(xs) => {
                if xs.len() == 0 {
                    Display::fmt("[]", f);
                    Ok(())
                } else {
                    Display::fmt("[", f);
                    let mut i = 0;
                    let mut res = Ok(());
                    while res.is_ok() && i < xs.len() {
                        res = xs[i].fmt(f);
                        Display::fmt(", ", f);
                        i += 1;
                    }
                    Display::fmt("]", f);
                    res
                }
            },
            Join::Disjoint(xs) => {
                if xs.len() == 0 {
                    Display::fmt("{}", f);
                    Ok(())
                } else {
                    Display::fmt("{", f);
                    let mut i = 0;
                    let mut res = Ok(());
                    while res.is_ok() && i < xs.len() {
                        res = xs[i].fmt(f);
                        Display::fmt(", ", f);
                        i += 1;
                    }
                    Display::fmt("}", f);
                    res
                }
            }
        }
    }
}

impl<T: Clone> Join<T> {
    pub fn unit(item: T) -> Join<T> {
        Join::Unit(item)
    }
    pub fn union(items: Vec<T>) -> Join<T> {
        Join::Union(items.into_iter().map(Join::Unit).map(Box::new).collect())
    }
    pub fn disjoint(items: Vec<T>) -> Join<T> {
        Join::Disjoint(items.into_iter().map(Join::Unit).map(Box::new).collect())
    }
}

impl<T> Named<T> {
    pub fn of<S: Into<String>>(name: S, item: T) -> Named<T> {
        Named {
            name: name.into(),
            item
        }
    }
}

impl<T: Debug> Debug for Named<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Named")
            .field("name", &self.name)
            .field("item", &self.item)
            .finish()
    }
}

impl<T, S: Into<String>> From<(S, T)> for Named<T> {
    fn from((name, item): (S, T)) -> Self {
        Named::of(name, item)
    }
}