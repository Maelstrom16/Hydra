/// Util struct for representing a usize address, usize bank pair.
pub struct BankedAddress<A, B> {
    pub address: A,
    pub bank: B
}