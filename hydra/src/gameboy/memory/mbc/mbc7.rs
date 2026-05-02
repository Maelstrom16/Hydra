use std::sync::{Arc, RwLock};

use sdl3::sensor::SensorType;

use crate::common::bit::BitVec;
use crate::common::errors::HydraIOError;
use crate::common::util::BankedAddress;
use crate::gamepad::ControllerState;
use crate::{deserialize, serialize};
use crate::gameboy::memory::{mbc, sram};
use crate::gameboy::memory::sram::Sram;
use crate::gameboy::memory::rom::{Rom, RomHeader};

pub struct MBC7 {
    rom: Rom<0x4000>,
    controllers: Arc<RwLock<ControllerState>>,

    ram_enables: [u8; 2],
    rom_bank: u8,

    latch_ready: bool,
    accel_x: u16,
    accel_y: u16,

    eeprom: Eeprom93LC56,
}

impl MBC7 {
    const ACCELEROMETER_RESET: u16 = 0x8000;
    const ACCELEROMETER_CENTER: u16 = 0x81D0;
    pub fn from_header(header: RomHeader, controllers: Arc<RwLock<ControllerState>>) -> Result<Self, HydraIOError> {
        Ok(MBC7 {
            rom: header.into_rom(),
            controllers,

            ram_enables: [0x00; 2],
            rom_bank: 1,
            
            latch_ready: true,
            accel_x: Self::ACCELEROMETER_RESET,
            accel_y: Self::ACCELEROMETER_RESET,

            eeprom: Eeprom93LC56::new()
        })
    }

    fn localize_rom_address(&self, address: u16) -> BankedAddress<u16, usize> {
        match address {
            0x0000..=0x3FFF => BankedAddress {address: address, bank: 0},
            0x4000..=0x7FFF => BankedAddress {address: address - self.rom.bank_size() as u16, bank: self.rom_bank as usize % self.rom.get_bank_count()},
            _ => unimplemented!("Attempted to localize invalid ROM address {}", address)
        }
    }

    fn is_ram_enabled(&self) -> bool {
        self.ram_enables == [0x0A, 0x40]
    }
}

impl mbc::MemoryBankController for MBC7 {
    fn read_rom_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        let BankedAddress { address, bank } = self.localize_rom_address(address);
        Ok(self.rom.read_bank(address, bank))
    }
    fn read_ram_u8(&self, address: u16) -> Result<u8, HydraIOError> {
        if self.is_ram_enabled() {
            Ok(match address & 0xF0F0 {
                0xA020 => self.accel_x.to_le_bytes()[0],
                0xA030 => self.accel_x.to_le_bytes()[1],
                0xA040 => self.accel_y.to_le_bytes()[0],
                0xA050 => self.accel_y.to_le_bytes()[1],
                0xA060 => 0x00,
                0xA070 => 0xFF,
                0xA080 => self.eeprom.poll_outputs(),
                0xA000..=0xA010 | 0xA090..=0xB0F0 => 0xFF,
                _ => unimplemented!("Attempted to read from invalid ROM address {:#06X}", address)
            })
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
    fn write_rom_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        Ok(match address {
            0x0000..=0x1FFF => {self.ram_enables[0] = value}
            0x2000..=0x3FFF => {self.rom_bank = value}
            0x4000..=0x5FFF => {self.ram_enables[1] = value}
            0x6000..=0x7FFF => { /* Do nothing */ }
            _ => unimplemented!("Attempted to write {:#04X} to invalid ROM address {:#06X}", value, address)
        })
    }
    fn write_ram_u8(&mut self, value: u8, address: u16) -> Result<(), HydraIOError> {
        if self.is_ram_enabled() {
            Ok(match address & 0xF0F0 {
                0xA000 => if value == 0x55 {
                    self.accel_x = Self::ACCELEROMETER_RESET;
                    self.accel_y = Self::ACCELEROMETER_RESET;
                    self.latch_ready = true;
                }
                0xA010 => if value == 0xAA && self.latch_ready {
                    let controller = self.controllers.read().unwrap();
                    let [accel_x_f32, accel_y_f32, accel_z_f32] = controller.poll_sensor(SensorType::Accelerometer);
                    self.accel_x = Self::ACCELEROMETER_CENTER.saturating_add_signed((accel_x_f32 * 11.0) as i16);
                    self.accel_y = Self::ACCELEROMETER_CENTER.saturating_add_signed((accel_z_f32 * 11.0) as i16);
                    println!("{:#06X}, {:#06X}", self.accel_x, self.accel_y);
                    self.latch_ready = false;
                }
                0xA080 => self.eeprom.update_inputs(value),
                0xA020..=0xA070 | 0xA090..=0xB0F0 => { /* Do nothing */ },
                _ => unimplemented!("Attempted to write {:#04X} to invalid RAM address {:#06X}", value, address)
            })
        } else {
            Err(HydraIOError::OpenBusAccess)
        }
    }
}

struct Eeprom93LC56 {
    chip_select: bool,
    clock: bool,
    data_in: bool,

    data_out: bool,

    state: EepromState,
    op_buffer: u8,
    address_buffer: u8,
    data_buffer: u16,

    write_enabled: bool,
    memory: [u16; 0x80]
}

impl Eeprom93LC56 {
    pub fn new() -> Self {
        Eeprom93LC56 { 
            chip_select: false, 
            clock: false, 
            data_in: false, 
            data_out: false,
            state: EepromState::Standby,
            op_buffer: 0,
            address_buffer: 0,
            data_buffer: 0,
            write_enabled: false, 
            memory: [0xFF; 0x80]
        }
    }

    fn update_inputs(&mut self, input: u8) {
        deserialize!(input;
            [7] as bool =>> (self.chip_select);
            [6] as bool =>> clock;
            [1] as bool =>> (self.data_in);
        );

        // On CLK rising edge
        if clock && !self.clock {
            match self.state {
                EepromState::Standby => if self.chip_select && self.data_in {
                    self.state = EepromState::Opcode(0)
                }
                EepromState::Opcode(ref mut cycles_elapsed) => {
                    self.op_buffer = (self.op_buffer << 1) | (self.data_in as u8);
                    *cycles_elapsed += 1;
                    if *cycles_elapsed == 2 {
                        self.state = EepromState::Address(0)
                    }
                }
                EepromState::Address(ref mut cycles_elapsed) => {
                    self.address_buffer = (self.address_buffer << 1) | (self.data_in as u8);
                    *cycles_elapsed += 1;
                    if *cycles_elapsed == 8 {
                        match self.op_buffer {
                            0b00 => match self.address_buffer >> 6 {
                                0b00 => {self.write_enabled = false; self.complete()}
                                0b01 => {self.state = EepromState::DataIn(0)}
                                0b10 => {self.write_all(0xFFFF)}
                                0b11 => {self.write_enabled = true; self.complete()}
                                _ => unimplemented!()
                            }
                            0b01 => {self.state = EepromState::DataIn(0)}
                            0b10 => {self.state = EepromState::DataOut(0)}
                            0b11 => {self.write(0xFFFF)}
                            _ => unimplemented!()
                        }
                    }
                }
                EepromState::DataIn(ref mut cycles_elapsed) => {
                    self.data_buffer = (self.data_buffer << 1) | (self.data_in as u16);
                    *cycles_elapsed += 1;
                    if *cycles_elapsed == 16 {
                        match self.op_buffer {
                            0b00 => {self.write_all(self.data_buffer)}
                            0b01 => {self.write(self.data_buffer)}
                            _ => unimplemented!()
                        }
                    }
                }
                EepromState::DataOut(ref mut cycles_elapsed) => {
                    self.data_out = self.memory[self.address_buffer as usize].test_bit(*cycles_elapsed as u16);
                    *cycles_elapsed += 1;
                    if *cycles_elapsed == 16 {
                        self.complete();
                    }
                }
            }
        }

        // Update clock when done
        self.clock = clock;
    }

    fn poll_outputs(&self) -> u8 {
        serialize!(
            0b11111110;
            (self.data_out as u8) =>> [0];
        )
    }

    fn write(&mut self, value: u16) {
        self.memory[self.address_buffer as usize] = value;
        self.complete();
        // TODO: Set status bit after internal delay
        self.data_out = true;
    }

    fn write_all(&mut self, value: u16) {
        self.memory.fill(value);
        self.complete();
        // TODO: Set status bit after internal delay
        self.data_out = true;
    }

    fn complete(&mut self) {
        // Update state
        self.state = EepromState::Standby;

        // Reset buffers
        self.op_buffer = 0;
        self.address_buffer = 0;
        self.data_buffer = 0;
    }
}

enum EepromState {
    Standby,
    Opcode(u8),
    Address(u8),
    DataIn(u8),
    DataOut(u8),
}