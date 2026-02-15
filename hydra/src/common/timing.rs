pub enum Delay {
    CyclesRemaining(usize),
    Idle
}

impl Delay {
    pub fn tick(&mut self) -> bool {
        match self {
            Delay::CyclesRemaining(n) if *n <= 1 => {
                *self = Delay::Idle;
                true
            }
            Delay::CyclesRemaining(n) => {
                *n -= 1;
                false
            }
            Delay::Idle => false
        }
    }

    pub fn queue(&mut self, cycles_until: usize) {
        *self = match *self {
            Delay::CyclesRemaining(current) if current <= cycles_until => return,
            _ => Delay::CyclesRemaining(cycles_until)
        } 
    }
}