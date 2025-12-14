mod mbc0;
// mod mbc1;
// mod mbc2;
// mod mbc3;
// mod mbc5;
// mod mbc6;
// mod mbc7;
// mod mmm01;
// mod huc1;
// mod huc3;
// mod pocketcamera;
// mod tama5;

use crate::common::errors::HydraIOError;
use crate::gameboy::memory;

pub trait MemoryBankController: Send + Sync + 'static {
    fn read_rom_u8(&self, address: usize) -> Result<u8, HydraIOError>;
    fn read_ram_u8(&self, address: usize) -> Result<u8, HydraIOError>;
    fn write_rom_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError>;
    fn write_ram_u8(&mut self, value: u8, address: usize) -> Result<(), HydraIOError>;
}

pub fn from_rom(rom: Box<[u8]>) -> Result<Box<dyn MemoryBankController>, HydraIOError> {
    match rom[memory::HARDWARE_ADDRESS] {
        0x00 | 0x08..=0x09 => Ok(Box::new(mbc0::MBC0::from_rom(rom)?)),
        0x01..=0x03 => panic!("MBC1 not yet supported"),
        0x05..=0x06 => panic!("MBC2 not yet supported"),
        0x0B..=0x0D => panic!("MMM01 not yet supported"),
        0x0F..=0x13 => panic!("MBC3 not yet supported"),
        0x19..=0x1E => panic!("MBC5 not yet supported"),
        0x20 => panic!("MBC6 not yet supported"),
        0x22 => panic!("MBC7 not yet supported"),
        0xFC => panic!("POCKET CAMERA not yet supported"),
        0xFD => panic!("TAMA5 not yet supported"),
        0xFE => panic!("HuC3 not yet supported"),
        0xFF => panic!("HuC1 not yet supported"),
        _ => Err(HydraIOError::MalformedROM("Undefined cartridge hardware identifier").into()),
    }
}

pub fn get_rom_size(rom: &Box<[u8]>) -> Result<usize, HydraIOError> {
    match rom[memory::ROM_SIZE_ADDRESS] {
        0x00 => Ok(0x008000 as usize), // 32 KiB
        0x01 => Ok(0x010000 as usize), // 64 KiB
        0x02 => Ok(0x020000 as usize), // 128 KiB
        0x03 => Ok(0x040000 as usize), // 256 KiB
        0x04 => Ok(0x080000 as usize), // 512 KiB
        0x05 => Ok(0x100000 as usize), // 1 MiB
        0x06 => Ok(0x200000 as usize), // 2 MiB
        0x07 => Ok(0x400000 as usize), // 4 MiB
        0x08 => Ok(0x800000 as usize), // 8 MiB
        0x52 => Ok(0x120000 as usize), // 1.1 MiB
        0x53 => Ok(0x140000 as usize), // 1.2 MiB
        0x54 => Ok(0x180000 as usize), // 1.5 MiB
        _ => Err(HydraIOError::MalformedROM("Undefined ROM size identifier").into()),
    }
}

pub fn get_ram_size(rom: &Box<[u8]>) -> Result<usize, HydraIOError> {
    match rom[memory::RAM_SIZE_ADDRESS] {
        0x00 => Ok(0x00000 as usize), // 0 KiB
        0x01 => Ok(0x00800 as usize), // 2 KiB
        0x02 => Ok(0x02000 as usize), // 8 KiB
        0x03 => Ok(0x08000 as usize), // 32 KiB
        0x04 => Ok(0x20000 as usize), // 128 KiB
        0x05 => Ok(0x10000 as usize), // 64 KiB
        _ => Err(HydraIOError::MalformedROM("Undefined RAM size identifier").into()),
    }
}
