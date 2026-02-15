use std::collections::VecDeque;

pub struct RenderQueue {
    fifo: VecDeque<Pixel>,
    obj_interrupts: Vec<u32>,
}

pub enum FetcherMode {
    Tile,
    DataLo,
    DataHi,
    Idle,
    Push,
}

impl RenderQueue {
    pub fn new() -> Self {
        RenderQueue {
            fifo: VecDeque::with_capacity(16),
            obj_interrupts: Vec::with_capacity(10),
        }
    }

    pub fn push(&mut self, pixel: Pixel) {
        self.fifo.push_back(pixel)
    }

    pub fn pop(&mut self) -> Pixel {
        self.fifo.pop_front().expect("Attempted to pop from the pixel FIFO while it was empty")
    }

    pub fn length(&self) -> usize {
        self.fifo.len()
    }
}

pub struct Pixel {
    color: u8,
}

//         MMIO::DMA => GBReg::new(match model {
//             Model::GameBoy(_) | Model::SuperGameBoy(_) => 0xFF,
//             Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => 0x00,
//         }, 0b11111111, 0b11111111),