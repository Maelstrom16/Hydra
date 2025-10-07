mod dmg;
//mod cgb;

use crate::common::errors::HydraIOError;
use crate::gameboy::Model;

pub trait ConsMemoryBankController {
    fn read_vram_u8(&self, address: usize) -> Result<u8, HydraIOError>;
    fn read_wram_u8(&self, address: usize) -> Result<u8, HydraIOError>;
    fn write_vram_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError>;
    fn write_wram_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError>;
}

pub fn from_model(model: Model) -> Result<Box<dyn ConsMemoryBankController>, HydraIOError> {
    match model {
        Model::GameBoy(_) | Model::SuperGameBoy(_) => Ok(Box::new(dmg::DMG::new()?)),
        Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => {
            panic!("CGB console memory not yet supported")
        }
    }
}
