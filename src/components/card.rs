type Defense = u8;
type Coin = u8;
type Authority = u8;
type Combat = u8;

struct Card {
    base: Option<Base> // None -> not a base, otherwise which base is it?

}

enum Base {
    Outpost(Defense),
    Base(Defense)
}

impl Base {
    pub fn is_outpost (&self) -> bool {
        match self {
            Base::Outpost(_) => true,
            _ => false
        }
    }
}

struct Good {
    trade: Coin,
    authority: Authority,
    combat: Combat
}

enum Either<L, R> {
    Left(L),
    Right(R)
}
