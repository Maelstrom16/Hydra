use hydra_macros::field_map;

#[field_map(u8)]
pub struct RegP1 {
    #[width(2)] _padding: !,
    #[width(1)] polling_buttons: bool,
    #[width(1)] polling_dpad: bool,
    #[width(1)] start_or_down: bool,
    #[width(1)] select_or_up: bool,
    #[width(1)] b_or_left: bool,
    #[width(1)] a_or_right: bool,
}

// TODO: Define layouts for stubs
#[field_map(u8)]
pub struct RegSb { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegSc { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegDiv { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegTima { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegTma { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegTac { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegIf { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr10 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr11 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr12 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr13 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr14 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr21 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr22 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr23 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr24 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr30 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr31 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr32 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr33 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr34 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr41 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr42 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr43 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr44 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr50 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr51 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegNr52 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav00 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav01 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav02 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav03 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav04 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav05 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav06 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav07 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav08 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav09 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav10 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav11 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav12 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav13 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav14 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWav15 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegLcdc {
    #[width(1)] ppu_enabled: bool,
    #[width(1)] win_map_index: u8,
    #[width(1)] win_enabled: bool,
    #[width(1)] tile_data_index: u8,
    #[width(1)] bg_map_index: u8,
    #[width(1)] obj_size: u8,
    #[width(1)] obj_enabled: bool,
    #[width(1)] tile_enabled_priority: bool,
}

#[field_map(u8)]
pub struct RegStat { 
    #[width(1)] _padding: !,
    #[width(1)] lyc_int: bool,
    #[width(1)] mode_2_int: bool,
    #[width(1)] mode_1_int: bool,
    #[width(1)] mode_0_int: bool,
    #[width(1)] ly_equals_lyc: bool,
    #[width(2)] ppu_mode: u8,
}

#[field_map(u8)]
pub struct RegScy { 
    #[width(8)] scy: u8
}

#[field_map(u8)]
pub struct RegScx { 
    #[width(8)] scx: u8
}

#[field_map(u8)]
pub struct RegLy { 
    #[width(8)] ly: u8
}

#[field_map(u8)]
pub struct RegLyc { 
    #[width(8)] lyc: u8
}

#[field_map(u8)]
pub struct RegDma { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegBgp { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegObp0 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegObp1 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWy { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegWx { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegKey0 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegKey1 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegVbk { 
    #[width(7)] _padding: !,
    #[width(1)] vbk: u8
}

#[field_map(u8)]
pub struct RegBoot { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegHdma1 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegHdma2 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegHdma3 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegHdma4 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegHdma5 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegRp { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegBcps { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegBcpd { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegOcps { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegOcpd { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegOpri { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegSvbk { 
    #[width(5)] _padding: !,
    #[width(3)] svbk: u8
}

#[field_map(u8)]
pub struct RegPcm12 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegPcm34 { #[width(8)] _stub: ! }

#[field_map(u8)]
pub struct RegIe { #[width(8)] _stub: ! }
