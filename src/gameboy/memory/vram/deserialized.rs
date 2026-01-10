use hydra_macros::DeserializedRegister8;

#[derive(DeserializedRegister8)]
pub struct TileAttributes {
    #[width(1)] pub priority: bool,
    #[width(1)] pub y_flip: bool,
    #[width(1)] pub x_flip: bool,
    #[width(1)] _padding: (),
    #[width(1)] pub bank: bool,
    #[width(3)] pub palette: u8,
}