pub struct Ime {
    ime: bool,
    queued: bool
}

impl Ime {
    pub fn get(&self) -> bool {
        self.ime
    }

    pub fn set(&mut self, val: bool) {
        self.ime = val
    }

    pub fn queue(&mut self) {
        self.queued = true
    }
    
    pub fn refresh(&mut self) {
        if self.queued {
            self.ime = true;
            self.queued = false;
        }
    }
}

impl Default for Ime {
    fn default() -> Self {
        Ime { ime: false, queued: false }
    }
}