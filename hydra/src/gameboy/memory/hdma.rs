use crate::{common::{errors::HydraIOError, timing::ModuloCounter}, deserialize, gameboy::{GbMode, memory::{MemoryMap, MemoryMapped}, ppu::{PpuMode, state::PpuState}}, serialize};

pub trait HdmAccessor {
    fn tick(&mut self, memory: &mut MemoryMap) -> bool;
    fn read(&self, address: u16) -> Result<u8, HydraIOError>;
    fn write(&mut self, val: u8, address: u16, ppu_state: &PpuState) -> Result<(), HydraIOError>;
}

pub fn from_mode(mode: &GbMode) -> Box<dyn HdmAccessor> {
    match mode {
        GbMode::DMG => Box::new(DmgHdmAccessor),
        GbMode::CGB => Box::new(CgbHdmAccessor::new()),
    }
}

pub struct DmgHdmAccessor;

impl HdmAccessor for DmgHdmAccessor {
    fn tick(&mut self, memory: &mut MemoryMap) -> bool {
        false // Do nothing
    }
    
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        Err(HydraIOError::OpenBusAccess)
    }

    fn write(&mut self, val: u8, address: u16, ppu_state: &PpuState) -> Result<(), HydraIOError> {
        Err(HydraIOError::OpenBusAccess)
    }
}

pub struct CgbHdmAccessor {
    source_addr: u16,
    dest_addr: u16,
    length: u8,
    transfer_type: HdmaType,
    row_counter: ModuloCounter<u8>
}

impl CgbHdmAccessor {
    fn new() -> Self {
        CgbHdmAccessor {
            source_addr: 0xFFF0,
            dest_addr: 0x1FF0,
            length: 0x7F,
            transfer_type: HdmaType::Complete,
            row_counter: ModuloCounter::new(0x00, 0x10)
        }
    }

    fn get_next_hblank(ppu_state: &PpuState) -> u8 {
        let ly = ppu_state.read_ly();
        if ly < 144 {ly} else {0}
    }
}

impl CgbHdmAccessor {
    fn read_hdma5(&self) -> u8 {
        serialize!(
            (self.transfer_type.as_u1()) =>> [7];
            (self.length) =>> [6..=0];
        )
    }

    fn write_hdma5(&mut self, val: u8, ppu_state: &PpuState) {
        deserialize!(val;
            [7] as bool =>> hblank_mode;
            [6..=0] =>> (self.length);
        );

        self.transfer_type = match hblank_mode {
            true => HdmaType::Hblank(Self::get_next_hblank(ppu_state)),
            false => HdmaType::General,
        }
    }
}

impl HdmAccessor for CgbHdmAccessor {
    fn tick(&mut self, memory: &mut MemoryMap) -> bool {
        let hdma_active_this_tick = match self.transfer_type {
            HdmaType::General => true,
            HdmaType::Hblank(next_hblank) => matches!(memory.ppu_state.get_mode(), PpuMode::HBlank) && next_hblank == memory.ppu_state.read_ly(),
            HdmaType::Complete => false,
        };

        if hdma_active_this_tick {
            let row_offset = self.row_counter.value as u16;
            let destination = self.dest_addr + 0x8000;
            let val = memory.read_u8(self.source_addr + row_offset, false);
            memory.write_u8(val, destination + row_offset);

            if self.row_counter.increment() {
                match self.length.checked_sub(1) {
                    Some(length_remaining) => {
                        if let HdmaType::Hblank(ref mut current_hblank) = self.transfer_type {
                            *current_hblank = Self::get_next_hblank(&memory.ppu_state);
                        }
                        self.source_addr += 0x10;
                        self.dest_addr += 0x10;
                        self.length = length_remaining;
                    }
                    None => {
                        self.transfer_type = HdmaType::Complete;
                        self.length = 0x7F;
                    }
                }
            }
        }

        hdma_active_this_tick
    }
    
    fn read(&self, address: u16) -> Result<u8, HydraIOError> {
        match address {
            0xFF51..=0xFF54 => Ok(0xFF),
            0xFF55 => Ok(self.read_hdma5()),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }

    fn write(&mut self, val: u8, address: u16, ppu_state: &PpuState) -> Result<(), HydraIOError> {
        match address {
            0xFF51 => Ok(self.source_addr = (self.source_addr & 0xFF) | ((val as u16) << 8)),
            0xFF52 => Ok(self.source_addr = (self.source_addr & 0xFF00) | (val as u16)),
            0xFF53 => Ok(self.dest_addr = (self.dest_addr & 0xFF) | ((val as u16 & 0x1F) << 8)),
            0xFF54 => Ok(self.dest_addr = (self.dest_addr & 0xFF00) | (val as u16)),
            0xFF55 => Ok(self.write_hdma5(val, ppu_state)),
            _ => Err(HydraIOError::OpenBusAccess)
        }
    }
}

enum HdmaType {
    General,
    Hblank(u8),
    Complete
}

impl HdmaType {
    pub(self) fn as_u1(&self) -> u8 {
        matches!(*self, Self::Complete) as u8
    }
}