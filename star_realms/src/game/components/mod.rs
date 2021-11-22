use std::ops::{AddAssign, Add};
use std::fmt::{Display, Formatter};
use ansi_term::Color;

pub mod card;
pub mod faction;
pub mod stack;

pub type Defense = u8;
pub type Coin = u8;
pub type Authority = u8;
pub type Combat = u8;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Goods {
    pub(crate) trade: Coin,
    pub(crate) authority: Authority,
    pub(crate) combat: Combat,
}

impl Goods {
    pub fn authority(auth: Authority) -> Goods {
        Goods {
            trade: 0,
            authority: auth,
            combat: 0
        }
    }
    pub fn trade(trade: Coin) -> Goods {
        Goods {
            trade,
            authority: 0,
            combat: 0
        }
    }
    pub fn combat(combat: Combat) -> Goods {
        Goods {
            combat,
            authority: 0,
            trade: 0
        }
    }
    pub fn none() -> Goods {
        Goods {
            combat: 0,
            authority: 0,
            trade: 0
        }
    }
}
impl Add for Goods {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Goods {
            trade: self.trade + rhs.trade,
            authority: self.authority + rhs.authority,
            combat: self.combat + rhs.combat,
        }
    }
}

impl AddAssign for Goods {
    fn add_assign(&mut self, rhs: Self) {
        self.trade += rhs.trade;
        self.authority += rhs.authority;
        self.combat += rhs.combat;
    }
}

impl Display for Goods {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}> <{}> <{}>",
               Color::Yellow.paint(self.trade.to_string()),
               Color::Blue.paint(self.authority.to_string()),
               Color::Red.paint(self.combat.to_string()))
    }
}