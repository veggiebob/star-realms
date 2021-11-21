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
}