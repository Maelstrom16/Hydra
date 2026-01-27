use hydra_macros::field_map;

#[field_map(u8)]
pub struct TileAttributes {
    #[range(7)] priority: bool,
    #[range(6)] y_flip: bool,
    #[range(5)] x_flip: bool,
    #[range(3)] bank_index: u8,
    #[range(2..=0)] palette: u8,
}