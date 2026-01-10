use hydra_macros::DeserializedRegister8;

#[derive(DeserializedRegister8)]
pub struct RegP1 {
    #[width(2)] _padding: (),
    #[width(1)] pub polling_buttons: bool,
    #[width(1)] pub polling_dpad: bool,
    #[width(1)] pub start_or_down: bool,
    #[width(1)] pub select_or_up: bool,
    #[width(1)] pub b_or_left: bool,
    #[width(1)] pub a_or_right: bool,
}

