use std::{cell::{Cell, RefCell}, rc::Rc};

use hydra_macros::field_map;

use crate::{common::timing::Delay, deserialize, gameboy::{GBRevision, Interrupt, InterruptFlags, Model, apu::Apu, memory::{MemoryMappedIo, io::MMIO}, ppu::state::PpuState}, serialize};

// TODO: Define layouts for stubs
#[field_map(u8)]
pub struct RegSb { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegSc { #[range(..)] _stub: u8 }

pub struct Divider { 
    div: Rc<Cell<u8>>,
}

impl Divider {
    fn new(model: Model) -> Self {
        Divider { 
            div: Rc::new(Cell::new(match model { 
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x18,
                Model::GameBoy(_) => 0xAB,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random()
            }))
        }
    }

    pub fn reset(&self) {
        self.div.set(0)
    }
}

pub struct MasterTimer {
    model: Rc<Model>,

    div_master: u16,
    tima: u8,
    tma: u8,
    tima_speed: TimaSpeed,
    tima_enabled: bool,
    timer_interrupt_status: InterruptStatus,

    system_speed: SystemSpeed,
    speed_switch_queued: bool,

    apu: Rc<RefCell<Apu>>,
    ppu_state: Rc<RefCell<PpuState>>,
    interrupt_flags: Rc<RefCell<InterruptFlags>>,
}

impl MasterTimer {
    pub fn new(model: Rc<Model>, apu: Rc<RefCell<Apu>>, ppu_state: Rc<RefCell<PpuState>>, interrupt_flags: Rc<RefCell<InterruptFlags>>) -> Self {
        MasterTimer { 
            div_master: match *model { 
                Model::GameBoy(Some(GBRevision::DMG0)) => 0x18,
                Model::GameBoy(_) => 0xAB,
                Model::SuperGameBoy(_) | Model::GameBoyColor(_) | Model::GameBoyAdvance(_) => rand::random(), // TODO: Number is supposed to be based on boot rom cycles,
            } << 6,
            model,
            
            tima: 0,
            tma: 0,
            tima_speed: TimaSpeed::Slowest,
            tima_enabled: false,
            timer_interrupt_status: InterruptStatus::Idle,

            system_speed: SystemSpeed::Standard,
            speed_switch_queued: false,

            apu,
            ppu_state,
            interrupt_flags,
        }
    }
    
    const MASTER_HZ: u32 = 4194304;
    const SYSTEM_HZ: u32 = Self::MASTER_HZ / 4;
    const DIV_HZ: u32 = Self::SYSTEM_HZ / 64;
    const PPU_DOTS_PER_FRAME: u32 = 70224;

    pub fn tick(&mut self) {
        self.update_div(self.div_master.wrapping_add(1));
        self.timer_interrupt_status = match self.timer_interrupt_status {
            InterruptStatus::Idle | InterruptStatus::Requesting => InterruptStatus::Idle,
            InterruptStatus::Queued => {
                self.tima = self.tma;
                self.interrupt_flags.borrow_mut().request(Interrupt::Timer);
                InterruptStatus::Requesting
            }
        };
        self.ppu_state.borrow_mut().tick();
    }

    pub fn get_ppu_dots(&self) -> u32 {
        self.ppu_state.borrow().get_dots()
    }

    pub fn refresh_tima_if_overflowing(&mut self) {
        if let InterruptStatus::Requesting = self.timer_interrupt_status {
            self.tima = self.tma;
        }
    }

    fn update_div(&mut self, new_div: u16) {
        let falling_edges = self.div_master & !new_div;
        if falling_edges & self.system_speed as u16 != 0 {
            self.apu.borrow_mut().tick();
        }
        if self.tima_enabled && falling_edges & self.tima_speed as u16 != 0 {
            self.tick_tima();
        }
    }

    fn tick_tima(&mut self) {
        let (tima, overflowed) = self.tima.overflowing_add(1);
        self.tima = tima;
        if overflowed {
            self.timer_interrupt_status = InterruptStatus::Queued;
        }
    }

    pub fn toggle_speed(&mut self) {
        if self.speed_switch_queued {
            self.speed_switch_queued = false;
            let new_system_speed = match self.system_speed {
                SystemSpeed::Standard => SystemSpeed::CgbDouble,
                SystemSpeed::CgbDouble => SystemSpeed::Standard,
            };

            // Tick APU timer if falling edge detected
            if (self.div_master & self.system_speed as u16 != 0) && (self.div_master & new_system_speed as u16 == 0) {
                self.apu.borrow_mut().tick();
            }
        }
    }
}

impl MemoryMappedIo<{MMIO::DIV as u16}> for MasterTimer {
    fn read(&self) -> u8 {
        (self.div_master >> 6) as u8 & 0xFF
    }
    fn write(&mut self, _val: u8) {
        self.update_div(0);
    }
}

impl MemoryMappedIo<{MMIO::TIMA as u16}> for MasterTimer {
    fn read(&self) -> u8 {
        self.tima
    }
    fn write(&mut self, val: u8) {
        self.tima = val;
        if let InterruptStatus::Queued = self.timer_interrupt_status {
            self.timer_interrupt_status = InterruptStatus::Idle;
        }
    }
}

impl MemoryMappedIo<{MMIO::TMA as u16}> for MasterTimer {
    fn read(&self) -> u8 {
        self.tma
    }
    fn write(&mut self, val: u8) {
        self.tma = val
    }
}

impl MemoryMappedIo<{MMIO::TAC as u16}> for MasterTimer {
    fn read(&self) -> u8 {
        serialize!(
            0b11111000;
            (self.tima_enabled as u8) =>> 2;
            (self.tima_speed.as_u2()) =>> 1..=0;
        )
    }
    fn write(&mut self, val: u8) {
        deserialize!(val;
            2 as bool =>> tima_enabled;
            1..=0 =>> tima_speed;
        );
        let tima_speed = TimaSpeed::from_u2(tima_speed);

        let old_div_bit_high = self.div_master & self.tima_speed as u16 != 0;
        let new_div_bit_low = self.div_master & tima_speed as u16 == 0;
        if match self.model.is_monochrome() {
            // On DMG, tick TIMA only when falling edge from (selected DIV bit && TIMA enable bit)
            // i.e. either selected bit went from set => unset, or TIMA was disabled
            true => self.tima_enabled && old_div_bit_high
                 && (!tima_enabled || new_div_bit_low),
            // On CGB, tick TIMA only when (falling edge from selected DIV bit && TIMA enable bit)
            // i.e. the selected went from set => unset, while TIMA was enabled
            // or, if TIMA was freshly enabled, the result is indeterminant
            false => old_div_bit_high && new_div_bit_low
                  && (self.tima_enabled || (tima_enabled && rand::random_bool(0.5)))
        } {
            self.tick_tima();
        }

        self.tima_speed = tima_speed;
        self.tima_enabled = tima_enabled;
    }
}

enum InterruptStatus {
    Idle,
    Queued,
    Requesting
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum SystemSpeed {
    Standard  = 0b010000000000,
    CgbDouble = 0b100000000000,
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum TimaSpeed {
    Fastest = 0b000000100,
    Fast    = 0b000010000,
    Slow    = 0b001000000,
    Slowest = 0b100000000,
}

impl TimaSpeed {
    const TIMA_SPEED_MAP: [TimaSpeed; 4] = [TimaSpeed::Slowest, TimaSpeed::Fastest, TimaSpeed::Fast, TimaSpeed::Slow];
    pub fn from_u2(val: u8) -> Self {
        Self::TIMA_SPEED_MAP[val as usize]
    }
    pub fn as_u2(&self) -> u8 {
        match self {
            Self::Slowest => 0b00,
            Self::Fastest => 0b01,
            Self::Fast => 0b10,
            Self::Slow => 0b11,
        }
    }
}

#[field_map(u8)]
pub struct RegNr10 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr11 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr12 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr13 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr14 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr21 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr22 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr23 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr24 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr30 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr31 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr32 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr33 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr34 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr41 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr42 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr43 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr44 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr50 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr51 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegNr52 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav00 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav01 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav02 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav03 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav04 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav05 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav06 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav07 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav08 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav09 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav10 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav11 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav12 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav13 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav14 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWav15 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegLcdc {
    #[range(7)] ppu_enabled: bool,
    #[range(6)] win_map_index: u8,
    #[range(5)] win_enabled: bool,
    #[range(4)] tile_data_index: u8,
    #[range(3)] bg_map_index: u8,
    #[range(2)] obj_size: u8,
    #[range(1)] obj_enabled: bool,
    #[range(0)] tile_enabled_priority: bool,
}

#[field_map(u8)]
pub struct RegStat { 
    #[range(6)] lyc_int: bool,
    #[range(5)] mode_2_int: bool,
    #[range(4)] mode_1_int: bool,
    #[range(3)] mode_0_int: bool,
    #[range(2)] ly_equals_lyc: bool,
    #[range(1..=0)] ppu_mode: u8,
}

#[field_map(u8)]
pub struct RegScy { 
    #[range(..)] scy: u8
}

#[field_map(u8)]
pub struct RegScx { 
    #[range(..)] scx: u8
}

#[field_map(u8)]
pub struct RegLy { 
    #[range(..)] ly: u8
}

#[field_map(u8)]
pub struct RegLyc { 
    #[range(..)] lyc: u8
}

#[field_map(u8)]
pub struct RegDma { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegBgp { 
    #[range(7..=6)] color3: u8,
    #[range(5..=4)] color2: u8,
    #[range(3..=2)] color1: u8,
    #[range(1..=0)] color0: u8,
}

impl RegBgp {
    pub fn get_color(&self, index: u8) -> &'static [u8] {
        let new_index = match index {
            0 => self.get_color0(),
            1 => self.get_color1(),
            2 => self.get_color2(),
            3 => self.get_color3(),
            _ => panic!("Invalid color index {} requested from BGP register", index)
        };

        // TODO: allow colors to be configured by user
        match new_index {
            0 => &[255, 255, 255, 255],
            1 => &[170, 170, 170, 255],
            2 => &[85, 85, 85, 255],
            3 => &[0, 0, 0, 255],
            _ => panic!("Invalid color index {} requested from BGP register", index)
        }
    }
}

#[field_map(u8)]
pub struct RegObp0 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegObp1 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWy { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegWx { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegKey0 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegKey1 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegVbk { 
    #[range(0)] vbk: u8
}

#[field_map(u8)]
pub struct RegBoot { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma1 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma2 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma3 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma4 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegHdma5 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegRp { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegBcps { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegBcpd { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegOcps { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegOcpd { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegOpri { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegSvbk { 
    #[range(2..=0)] svbk: u8
}

#[field_map(u8)]
pub struct RegPcm12 { #[range(..)] _stub: u8 }

#[field_map(u8)]
pub struct RegPcm34 { #[range(..)] _stub: u8 }