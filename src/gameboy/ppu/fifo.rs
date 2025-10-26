use std::collections::VecDeque;

pub struct RenderQueue {
    fifo: VecDeque<Pixel>,
    obj_interrupts: Vec<u32>
}

pub enum FetcherMode {
    Tile,
    DataLo,
    DataHi,
    Idle,
    Push
}

impl RenderQueue {
    pub fn new() -> Self {
        RenderQueue { 
            fifo: VecDeque::with_capacity(16),
            obj_interrupts: Vec::with_capacity(10)
        }
    }

    pub fn step_fifo(&mut self) {

    }

    pub fn step_fetcher(&mut self) {

    }
}

struct Pixel {
    color: u8
}