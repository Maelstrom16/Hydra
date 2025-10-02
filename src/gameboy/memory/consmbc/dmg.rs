use crate::common::errors::HydraIOError;
use crate::gameboy::memory::consmbc;

pub struct DMG {
    vram: Vec<u8>,
    wram: Vec<u8>,
}

impl DMG {
    pub fn new() -> Result<Self, HydraIOError> {
        Ok(DMG {
            vram: vec![0; 0x2000],
            wram: vec![0; 0x2000],
        })
    }
}

impl consmbc::ConsMemoryBankController for DMG {
    fn read_vram_u8(&self, address: usize) -> Result<u8, HydraIOError> {
        Ok(self.vram[address])
    }
    fn read_wram_u8(&self, address: usize) -> Result<u8, HydraIOError> {
        Ok(self.wram[address])
    }
    fn write_vram_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError> {
        self.vram[address] = value;
        Ok(())
    }
    fn write_wram_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError> {
        self.wram[address] = value;
        Ok(())
    }
}
