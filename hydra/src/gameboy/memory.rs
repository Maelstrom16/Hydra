mod mbc;
pub mod oam;
pub mod rom;
mod sram;
pub mod vram;
pub mod wram;

use crate::{
    common::errors::HydraIOError,
    gameboy::{
        InterruptEnable, InterruptFlags, Joypad, Model, apu::{Apu, channel::{Noise, Pulse, Wave}}, memory::{oam::Oam, rom::Rom, vram::Vram, wram::Wram}, ppu::{colormap::ColorMap, lcdc::LcdController, state::PpuState}, serial::SerialConnection, timer::MasterTimer
    },
};
use std::{cell::{Cell, RefCell}, fs, path::Path, rc::Rc, time::Duration};

pub struct MemoryMap {
    cartridge: Option<Box<dyn mbc::MemoryBankController>>, // ROM, SRAM
    vram: Rc<RefCell<Vram>>,
    wram: Rc<RefCell<Wram>>,
    oam: Rc<RefCell<Oam>>,
    hram: [u8; 0x7F],

    joypad: Rc<RefCell<Joypad>>,
    serial: SerialConnection,
    timer: Rc<RefCell<MasterTimer>>,
    interrupt_flags: Rc<RefCell<InterruptFlags>>,
    pulse1: Rc<RefCell<Pulse>>,
    pulse2: Rc<RefCell<Pulse>>,
    wave: Rc<RefCell<Wave>>,
    noise: Rc<RefCell<Noise>>,
    lcd_controller: Rc<RefCell<LcdController>>,
    ppu_state: Rc<RefCell<PpuState>>,
    scy: Rc<Cell<u8>>,
    scx: Rc<Cell<u8>>,
    color_map: Rc<RefCell<ColorMap>>,
    wy: Rc<Cell<u8>>,
    wx: Rc<Cell<u8>>,
    interrupt_enable: Rc<RefCell<InterruptEnable>>,

    dma_source: u8,
    dma_cycle: Option<u8>,
}

impl MemoryMap {
    pub fn new(model: &Rc<Model>, rom: Rom, vram: Rc<RefCell<Vram>>, wram: Rc<RefCell<Wram>>, oam: Rc<RefCell<Oam>>, joypad: Rc<RefCell<Joypad>>, timer: Rc<RefCell<MasterTimer>>, interrupt_flags: Rc<RefCell<InterruptFlags>>, apu: Rc<RefCell<Apu>>, lcd_controller: Rc<RefCell<LcdController>>, ppu_state: Rc<RefCell<PpuState>>, scy: Rc<Cell<u8>>, scx: Rc<Cell<u8>>, color_map: Rc<RefCell<ColorMap>>, wy: Rc<Cell<u8>>, wx: Rc<Cell<u8>>, interrupt_enable: Rc<RefCell<InterruptEnable>>) -> Result<MemoryMap, HydraIOError> {
        let (pulse1, pulse2, wave, noise) = apu.borrow().clone_pointers();
        Ok(MemoryMap {
            cartridge: Some(rom.into_mbc()?),
            vram,
            wram,
            oam,
            hram: [0; 0x7F],

            joypad,
            serial: SerialConnection::new(model.clone()),
            timer,
            interrupt_flags,
            pulse1,
            pulse2,
            wave,
            noise,
            lcd_controller,
            ppu_state,
            scy,
            scx,
            color_map,
            wy,
            wx,
            interrupt_enable,

            dma_source: match model.is_monochrome() {
                true => 0xFF,
                false => 0x00,
            },
            dma_cycle: None,
        })
    }

    pub fn hot_swap_rom(&mut self, path: &Path) -> Result<(), HydraIOError> {
        let rom = fs::read(path)?;
        self.cartridge = Some(Rom::from_vec(rom)?.into_mbc()?);
        Ok(())
    }

    pub fn tick_dma(&mut self) {
        if let Some(cycle) = self.dma_cycle {
            let source_address = (self.dma_source as u16) << 8 | cycle as u16;
            let destination_address = 0xFE00 | cycle as u16;
            self.write_u8(self.read_u8(source_address, true), destination_address, true);

            let next_dma_cycle = cycle + 1;
            self.dma_cycle = if next_dma_cycle < 160 {
                Some(next_dma_cycle)
            } else {
                None
            };
        }
    }

    pub fn read_u8(&self, address: u16, is_dma: bool) -> u8 {
        let mem_accessible = is_dma || matches!(self.dma_cycle, None);
        let read_result = if mem_accessible { match address {
            0x0000..=0x7FFF => self.cartridge.as_ref().map(|this| this.read_rom_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF => self.vram.borrow().read_u8(address),
            0xA000..=0xBFFF => self.cartridge.as_ref().map(|this| this.read_ram_u8(address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF => Ok(self.wram.borrow().read_u8(address)),
            0xE000..=0xFDFF => Ok(self.wram.borrow().read_u8(address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.borrow().read(address),
            0xFF00 => Ok(self.joypad.borrow().read_joyp()),
            0xFF01 => Ok(self.serial.read_sb()),
            0xFF02 => Ok(self.serial.read_sc()),
            0xFF04 => Ok(self.timer.borrow().read_div()),
            0xFF05 => Ok(self.timer.borrow().read_tima()),
            0xFF06 => Ok(self.timer.borrow().read_tma()),
            0xFF07 => Ok(self.timer.borrow().read_tac()),
            0xFF0F => Ok(self.interrupt_flags.borrow().read_if()),
            0xFF10 => Ok(self.pulse1.borrow().read_nr10()),
            0xFF11 => Ok(self.pulse1.borrow().read_nrx1()),
            0xFF12 => Ok(self.pulse1.borrow().read_nrx2()),
            0xFF13 => Ok(self.pulse1.borrow().read_nrx3()),
            0xFF14 => Ok(self.pulse1.borrow().read_nrx4()),
            0xFF16 => Ok(self.pulse2.borrow().read_nrx1()),
            0xFF17 => Ok(self.pulse2.borrow().read_nrx2()),
            0xFF18 => Ok(self.pulse2.borrow().read_nrx3()),
            0xFF19 => Ok(self.pulse2.borrow().read_nrx4()),
            0xFF1A => Ok(self.wave.borrow().read_nr30()),
            0xFF1B => Ok(self.wave.borrow().read_nr31()),
            0xFF1C => Ok(self.wave.borrow().read_nr32()),
            0xFF1D => Ok(self.wave.borrow().read_nr33()),
            0xFF1E => Ok(self.wave.borrow().read_nr34()),
            0xFF20 => Ok(self.noise.borrow().read_nr41()),
            0xFF21 => Ok(self.noise.borrow().read_nr42()),
            0xFF22 => Ok(self.noise.borrow().read_nr43()),
            0xFF23 => Ok(self.noise.borrow().read_nr44()),
            0xFF30..=0xFF3F => Ok(self.wave.borrow().read_waveram(address as usize - 0xFF30)),
            0xFF40 => Ok(self.lcd_controller.borrow().read_lcdc()),
            0xFF41 => Ok(self.ppu_state.borrow().read_stat()),
            0xFF42 => Ok(self.scy.get()),
            0xFF43 => Ok(self.scx.get()),
            0xFF44 => Ok(self.ppu_state.borrow().read_ly()),
            0xFF45 => Ok(self.ppu_state.borrow().read_lyc()),
            0xFF46 => Ok(self.dma_source),
            0xFF47 => Ok(self.color_map.borrow().read_bgp()),
            0xFF48 => Ok(self.color_map.borrow().read_obp(0)),
            0xFF49 => Ok(self.color_map.borrow().read_obp(1)),
            0xFF4A => Ok(self.wy.get()),
            0xFF4B => Ok(self.wx.get()),
            0xFF4D => Ok(self.timer.borrow().read_key1()),
            0xFF4F => Ok(self.vram.borrow().read_vbk()),
            0xFF70 => Ok(self.wram.borrow().read_wbk()),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            0xFFFF => Ok(self.interrupt_enable.borrow().read_ie()),
            _ => Err(HydraIOError::OpenBusAccess)
        }} else { match address {
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80]),
            _ => Err(HydraIOError::OpenBusAccess)
        }};

        match read_result {
            Ok(value) => value,
            Err(HydraIOError::OpenBusAccess) => 0xFF, //println!("Warning: Read from open bus at address {:#06X}", address),
            Err(e) => panic!("Error reading from memory.\n{}", e),
        }
    }

    pub fn write_u8(&mut self, value: u8, address: u16, is_dma: bool) -> () {
        let mem_accessible = is_dma || matches!(self.dma_cycle, None);
        let write_result = if mem_accessible { match address {
            0x0000..=0x7FFF => self.cartridge.as_mut().map(|this| this.write_rom_u8(value, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0x8000..=0x9FFF => self.vram.borrow_mut().write_u8(value, address),
            0xA000..=0xBFFF => self.cartridge.as_mut().map(|this| this.write_ram_u8(value, address)).ok_or(HydraIOError::OpenBusAccess).flatten(),
            0xC000..=0xDFFF => Ok(self.wram.borrow_mut().write_u8(value, address)),
            0xE000..=0xFDFF => Ok(self.wram.borrow_mut().write_u8(value, address - 0x2000)), // Treat exactly like WRAM
            0xFE00..=0xFEFF => self.oam.borrow_mut().write(address, value),
            0xFF00 => Ok(self.joypad.borrow_mut().write_joyp(value)),
            0xFF01 => Ok(self.serial.write_sb(value)),
            0xFF02 => Ok(self.serial.write_sc(value)),
            0xFF04 => Ok(self.timer.borrow_mut().write_div(value)),
            0xFF05 => Ok(self.timer.borrow_mut().write_tima(value)),
            0xFF06 => Ok(self.timer.borrow_mut().write_tma(value)),
            0xFF07 => Ok(self.timer.borrow_mut().write_tac(value)),
            0xFF0F => Ok(self.interrupt_flags.borrow_mut().write_if(value)),
            0xFF10 => Ok(self.pulse1.borrow_mut().write_nr10(value)),
            0xFF11 => Ok(self.pulse1.borrow_mut().write_nrx1(value)),
            0xFF12 => Ok(self.pulse1.borrow_mut().write_nrx2(value)),
            0xFF13 => Ok(self.pulse1.borrow_mut().write_nrx3(value)),
            0xFF14 => Ok(self.pulse1.borrow_mut().write_nrx4(value)),
            0xFF16 => Ok(self.pulse2.borrow_mut().write_nrx1(value)),
            0xFF17 => Ok(self.pulse2.borrow_mut().write_nrx2(value)),
            0xFF18 => Ok(self.pulse2.borrow_mut().write_nrx3(value)),
            0xFF19 => Ok(self.pulse2.borrow_mut().write_nrx4(value)),
            0xFF1A => Ok(self.wave.borrow_mut().write_nr30(value)),
            0xFF1B => Ok(self.wave.borrow_mut().write_nr31(value)),
            0xFF1C => Ok(self.wave.borrow_mut().write_nr32(value)),
            0xFF1D => Ok(self.wave.borrow_mut().write_nr33(value)),
            0xFF1E => Ok(self.wave.borrow_mut().write_nr34(value)),
            0xFF20 => Ok(self.noise.borrow_mut().write_nr41(value)),
            0xFF21 => Ok(self.noise.borrow_mut().write_nr42(value)),
            0xFF22 => Ok(self.noise.borrow_mut().write_nr43(value)),
            0xFF23 => Ok(self.noise.borrow_mut().write_nr44(value)),
            0xFF30..=0xFF3F => Ok(self.wave.borrow_mut().write_waveram(value, address as usize - 0xFF30)),
            0xFF40 => Ok(self.lcd_controller.borrow_mut().write_lcdc(value)),
            0xFF41 => Ok(self.ppu_state.borrow_mut().write_stat(value)),
            0xFF42 => Ok(self.scy.set(value)),
            0xFF43 => Ok(self.scx.set(value)),
            0xFF44 => Ok(self.ppu_state.borrow_mut().write_ly(value)),
            0xFF45 => Ok(self.ppu_state.borrow_mut().write_lyc(value)),
            0xFF46 => Ok({self.dma_source = value; self.dma_cycle = Some(0);}),
            0xFF47 => Ok(self.color_map.borrow_mut().write_bgp(value)),
            0xFF48 => Ok(self.color_map.borrow_mut().write_obp(value, 0)),
            0xFF49 => Ok(self.color_map.borrow_mut().write_obp(value, 1)),
            0xFF4A => Ok(self.wy.set(value)),
            0xFF4B => Ok(self.wx.set(value)),
            0xFF4D => Ok(self.timer.borrow_mut().write_key1(value)),
            0xFF4F => Ok(self.vram.borrow_mut().write_vbk(value)),
            0xFF70 => Ok(self.wram.borrow_mut().write_wbk(value)),
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = value),
            0xFFFF => Ok(self.interrupt_enable.borrow_mut().write_ie(value)),
            _ => Err(HydraIOError::OpenBusAccess)
        }} else { match address {
            0xFF80..=0xFFFE => Ok(self.hram[address as usize - 0xFF80] = value),
            _ => Err(HydraIOError::OpenBusAccess)
        }};
        match write_result {
            Ok(_) => {}
            Err(HydraIOError::OpenBusAccess) => {}//println!("Warning: Write to open bus at address {:#06X}", address),
            Err(e) => panic!("Error writing to memory.\n{}", e)
        }
    }
}