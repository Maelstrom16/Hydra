use crate::gameboy::{
    Model,
    memory::{self, TITLE_ADDRESS},
};

#[derive(Default)]
pub struct CPU {
    af: [u8; 2],
    bc: [u8; 2],
    de: [u8; 2],
    hl: [u8; 2],
    sp: u16,
    pc: u16,
}

impl CPU {
    pub fn from_rom_and_model(rom: &Vec<u8>, model: Model) -> Self {
        const sp: u16 = 0xFFFE;
        const pc: u16 = 0x0100;
        match model {
            Model::DMG0 => CPU {
                af: [0x01, 0b0000 << 4],
                bc: [0xFF, 0x13],
                de: [0x00, 0xC1],
                hl: [0x84, 0x03],
                sp,
                pc,
            },
            Model::DMG => CPU {
                af: [
                    0x01,
                    if rom[memory::HEADER_CHECKSUM_ADDRESS] == 0 {
                        0b1000 << 4
                    } else {
                        0b1011 << 4
                    },
                ],
                bc: [0x00, 0x13],
                de: [0x00, 0xD8],
                hl: [0x01, 0x4D],
                sp,
                pc,
            },
            Model::MGB => CPU {
                af: [
                    0xFF,
                    if rom[memory::HEADER_CHECKSUM_ADDRESS] == 0 {
                        0b1000 << 4
                    } else {
                        0b1011 << 4
                    },
                ],
                bc: [0x00, 0x13],
                de: [0x00, 0xD8],
                hl: [0x01, 0x4D],
                sp,
                pc,
            },
            Model::SGB => CPU {
                af: [0x01, 0b0000 << 4],
                bc: [0x00, 0x14],
                de: [0x00, 0x00],
                hl: [0xC0, 0x60],
                sp,
                pc,
            },
            Model::SGB2 => CPU {
                af: [0xFF, 0b0000 << 4],
                bc: [0x00, 0x14],
                de: [0x00, 0x00],
                hl: [0xC0, 0x60],
                sp,
                pc,
            },
            Model::CGBdmg => {
                let mut b = 0x00;
                let mut hl = [0x00, 0x7C];
                if rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x01
                    || rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x33
                        && rom[memory::NEW_LICENSEE_CODE_ADDRESS] == 0x01
                {
                    for offset in 0..16 {
                        // If either licensee code is 0x01, B = sum of all title bytes
                        b += rom[TITLE_ADDRESS + offset];
                    }
                    if b == 0x43 || b == 0x58 {
                        // And, check special cases for HL
                        hl = [0x99, 0x1A];
                    }
                }
                CPU {
                    af: [0x11, 0b1000 << 4],
                    bc: [b, 0x00],
                    de: [0x00, 0x08],
                    hl,
                    sp,
                    pc,
                }
            }
            Model::AGBdmg => {
                let mut b = 0x01;
                let mut hl = [0x00, 0x7C];
                let mut f = 0b00000000;
                if rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x01
                    || rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x33
                        && rom[memory::NEW_LICENSEE_CODE_ADDRESS] == 0x01
                {
                    for offset in 0..16 {
                        // If either licensee code is 0x01, B = sum of all title bytes
                        b += rom[TITLE_ADDRESS + offset];
                    }
                    if b & 0b1111 == 0 {
                        // Last op is an INC; set h flag...
                        f |= 0b0010 << 4;
                        if b == 0 {
                            // ...and z flag if necessary
                            f |= 0b1000 << 4
                        }
                    } else if b == 0x44 || b == 0x59 {
                        // Otherwise, still check special cases for HL
                        hl = [0x99, 0x1A];
                    }
                }
                CPU {
                    af: [0x11, f],
                    bc: [b, 0x00],
                    de: [0x00, 0x08],
                    hl,
                    sp,
                    pc,
                }
            }
            Model::CGB => CPU {
                af: [0x11, 0b1000 << 4],
                bc: [0x00, 0x00],
                de: [0xFF, 0x56],
                hl: [0x00, 0x0D],
                sp,
                pc,
            },
            Model::AGB => CPU {
                af: [0x11, 0b0000 << 4],
                bc: [0x01, 0x00],
                de: [0xFF, 0x56],
                hl: [0x00, 0x0D],
                sp,
                pc,
            },
        }
    }

    pub fn step(&mut self, memory: &memory::Memory) -> () {
        let opcode_address = self.pc;
        self.pc += 1;
    }
}
