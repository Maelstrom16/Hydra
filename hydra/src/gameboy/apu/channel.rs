// use std::{cell::Cell, rc::Rc};

// use crate::gameboy::memory::io::MMIO;

// pub struct Pulse1 {
//     nr10: Rc<Cell<u8>>,
//     nr11: Rc<Cell<u8>>,
//     nr12: Rc<Cell<u8>>,
//     nr13: Rc<Cell<u8>>,
//     nr14: Rc<Cell<u8>>,
// }

// impl Pulse1 {
//     fn new(io: &IoMap) -> Self {
//         Pulse1 { 
//             nr10: io.clone_register(MMIO::NR10), 
//             nr11: io.clone_register(MMIO::NR11), 
//             nr12: io.clone_register(MMIO::NR12), 
//             nr13: io.clone_register(MMIO::NR13), 
//             nr14: io.clone_register(MMIO::NR14)
//         }
//     }
// }

//         MMIO::NR10 => GBReg::new(0x80, 0b01111111, 0b01111111),
//         MMIO::NR11 => GBReg::new(0xBF, 0b11000000, 0b11111111),
//         MMIO::NR12 => GBReg::new(0xF3, 0b11111111, 0b11111111),
//         MMIO::NR13 => GBReg::new(0xFF, 0b00000000, 0b11111111),
//         MMIO::NR14 => GBReg::new(0xBF, 0b01000000, 0b11000111),
//         MMIO::NR21 => GBReg::new(0x3F, 0b11000000, 0b11111111),
//         MMIO::NR22 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::NR23 => GBReg::new(0xFF, 0b00000000, 0b11111111),
//         MMIO::NR24 => GBReg::new(0xBF, 0b01000000, 0b11000111),
//         MMIO::NR30 => GBReg::new(0x7F, 0b10000000, 0b10000000),
//         MMIO::NR31 => GBReg::new(0xFF, 0b00000000, 0b11111111),
//         MMIO::NR32 => GBReg::new(0x9F, 0b01100000, 0b01100000),
//         MMIO::NR33 => GBReg::new(0xFF, 0b11111111, 0b11111111),
//         MMIO::NR34 => GBReg::new(0xBF, 0b01000000, 0b11000111),
//         MMIO::NR41 => GBReg::new(0xFF, 0b00000000, 0b00111111),
//         MMIO::NR42 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::NR43 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::NR44 => GBReg::new(0xBF, 0b01000000, 0b11000000),
//         MMIO::NR50 => GBReg::new(0x77, 0b11111111, 0b11111111),
//         MMIO::NR51 => GBReg::new(0xF3, 0b11111111, 0b11111111),
//         MMIO::NR52 => GBReg::new(match model {
//             Model::GameBoy(_) => 0xF1,
//             Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0xF0,
//         }, 0b10001111, 0b00001111),
//         MMIO::WAV00 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV01 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV02 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV03 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV04 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV05 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV06 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV07 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV08 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV09 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV10 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV11 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV12 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV13 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV14 => GBReg::new(0x00, 0b11111111, 0b11111111),
//         MMIO::WAV15 => GBReg::new(0x00, 0b11111111, 0b11111111),

//         MMIO::PCM12 => match model { 
//             Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
//             Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000), // TODO: Verify startup value
//         },
//         MMIO::PCM34 => match model { 
//             Model::GameBoy(_) | Model::SuperGameBoy(_) => GBReg::new_unimplemented(),
//             Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => GBReg::new(0xFF, 0b11111111, 0b00000000), // TODO: Verify startup value
//         },