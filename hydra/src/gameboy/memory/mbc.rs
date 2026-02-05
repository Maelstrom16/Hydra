pub mod mbc0;
pub mod mbc1;
// pub mod mbc2;
// pub mod mbc3;
// pub mod mbc5;
// pub mod mbc6;
// pub mod mbc7;
// pub mod mmm01;
// pub mod huc1;
// pub mod huc3;
// pub mod pocketcamera;
// pub mod tama5;

use crate::common::errors::HydraIOError;

pub trait MemoryBankController: Send + Sync + 'static {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError>;
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError>;
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError>;
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError>;
}