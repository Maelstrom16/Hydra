/// Util struct for representing a usize address, usize bank pair.
pub struct BankedAddress<A, B> {
    pub address: A,
    pub bank: B
}

/// Util struct for representing a 2D point or offset.
pub struct Coords<T> {
    pub x: T,
    pub y: T,
}