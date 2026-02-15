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