use hydra_macros::field_map;

#[field_map(u8)]
pub struct RegP1 {
    #[range(5)] polling_buttons: bool,
    #[range(4)] polling_dpad: bool,
    #[range(3)] start_or_down: bool,
    #[range(2)] select_or_up: bool,
    #[range(1)] b_or_left: bool,
    #[range(0)] a_or_right: bool,
}

// TODO: Define layouts for stubs
#[field_map(u8)]
pub struct RegSb { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegSc { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegDiv { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegTima { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegTma { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegTac { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegIf { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr10 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr11 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr12 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr13 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr14 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr21 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr22 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr23 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr24 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr30 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr31 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr32 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr33 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr34 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr41 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr42 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr43 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr44 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr50 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr51 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr52 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav00 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav01 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav02 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav03 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav04 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav05 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav06 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav07 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav08 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav09 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav10 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav11 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav12 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav13 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav14 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav15 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegLcdc {
    #[range(7)] ppu_enabled: bool,
    #[range(6)] win_map_index: u8,
    #[range(5)] win_enabled: bool,
    #[range(4)] tile_data_index: u8,
    #[range(3)] bg_map_index: u8,
    #[range(2)] obj_size: u8,
    #[range(1)] obj_enabled: bool,
    #[range(0)] tile_enabled_priority: bool,
}

#[field_map(u8)]
pub struct RegStat { 
    #[range(6)] lyc_int: bool,
    #[range(5)] mode_2_int: bool,
    #[range(4)] mode_1_int: bool,
    #[range(3)] mode_0_int: bool,
    #[range(2)] ly_equals_lyc: bool,
    #[range(1..=0)] ppu_mode: u8,
}

#[field_map(u8)]
pub struct RegScy { 
    #[range(..)] scy: u8
}

#[field_map(u8)]
pub struct RegScx { 
    #[range(..)] scx: u8
}

#[field_map(u8)]
pub struct RegLy { 
    #[range(..)] ly: u8
}

#[field_map(u8)]
pub struct RegLyc { 
    #[range(..)] lyc: u8
}

#[field_map(u8)]
pub struct RegDma { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegBgp { 
    #[range(7..=6)] color3: u8,
    #[range(5..=4)] color2: u8,
    #[range(3..=2)] color1: u8,
    #[range(1..=0)] color0: u8,
}

#[field_map(u8)]
pub struct RegObp0 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegObp1 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWy { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWx { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegKey0 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegKey1 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegVbk { 
    #[range(0)] vbk: u8
}

#[field_map(u8)]
pub struct RegBoot { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma1 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma2 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma3 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma4 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma5 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegRp { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegBcps { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegBcpd { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegOcps { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegOcpd { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegOpri { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegSvbk { 
    #[range(2..=0)] svbk: u8
}

#[field_map(u8)]
pub struct RegPcm12 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegPcm34 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegIe { #[range(..)] _stub: u8 }
