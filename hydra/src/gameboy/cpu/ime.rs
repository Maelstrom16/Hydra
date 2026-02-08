pub struct InterruptHandler {
    pub ime: bool,
    cycles_until_ime: Option<u8>,
    pub halted: bool,
    cycles_until_halt_bug: Option<u8>
}

impl InterruptHandler {
    pub fn queue_ime(&mut self) {
        self.cycles_until_ime = Some(2)
    }
    pub fn cancel_ime(&mut self) {
        self.cycles_until_ime = None
    }

    pub fn queue_halt_bug(&mut self) {
        self.cycles_until_halt_bug = Some(2)
    }
    
    pub fn refresh(&mut self, pc: &mut u16, pc_old: u16) {
        let decrements_to_zero = |n: &mut u8| {*n -= 1; *n == 0};
        if let Some(_) = self.cycles_until_ime.take_if(decrements_to_zero) {self.ime = true;}
        if let Some(_) = self.cycles_until_halt_bug.take_if(decrements_to_zero) {*pc = pc_old;}
    }
}

impl Default for InterruptHandler {
    fn default() -> Self {
        InterruptHandler { ime: false, cycles_until_ime: None, halted: false, cycles_until_halt_bug: None }
    }
}