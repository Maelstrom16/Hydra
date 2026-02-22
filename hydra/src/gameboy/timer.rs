use std::{cell::RefCell, rc::Rc};

use crate::{common::timing::ModuloCounter, deserialize, gameboy::{GBRevision, Interrupt, InterruptFlags, Model, apu::Apu, ppu::state::PpuState}, serialize};

pub struct MasterTimer {
    model: Rc<Model>,

    machine_cycle_timer: ModuloCounter<u8>,
    div_full: u16,
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
            machine_cycle_timer: ModuloCounter::new(0, 4),
            div_full: match *model { 
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
    pub const PPU_DOTS_PER_FRAME: u32 = 70224;

    pub fn tick(&mut self) {
        if self.machine_cycle_timer.increment() {
            self.update_div(self.div_full.wrapping_add(1));

            self.timer_interrupt_status = match self.timer_interrupt_status {
                InterruptStatus::Idle | InterruptStatus::Requesting => InterruptStatus::Idle,
                InterruptStatus::Queued => {
                    self.tima = self.tma;
                    self.interrupt_flags.borrow_mut().request(Interrupt::Timer);
                    InterruptStatus::Requesting
                }
            };
        }

        self.ppu_state.borrow_mut().tick();
    }

    pub fn is_system_cycle(&self) -> bool {
        self.machine_cycle_timer.has_completed_cycle()
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
        let falling_edges = self.div_full & !new_div;
        if falling_edges & self.system_speed as u16 != 0 {
            self.apu.borrow_mut().apu_tick();
        }
        if self.tima_enabled && falling_edges & self.tima_speed as u16 != 0 {
            self.tick_tima();
        }
        // Update DIV when done
        self.div_full = new_div;
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
            if (self.div_full & self.system_speed as u16 != 0) && (self.div_full & new_system_speed as u16 == 0) {
                self.apu.borrow_mut().apu_tick();
            }
        }
    }

    pub fn is_speed_switch_requested(&self) -> bool {
        self.speed_switch_queued
    }
}

impl MasterTimer {
    pub fn read_div(&self) -> u8 {
        (self.div_full >> 6) as u8 & 0xFF
    }
    pub fn write_div(&mut self, _val: u8) {
        self.update_div(0);
    }
    
    pub fn read_tima(&self) -> u8 {
        self.tima
    }
    pub fn write_tima(&mut self, val: u8) {
        self.tima = val;
        if let InterruptStatus::Queued = self.timer_interrupt_status {
            self.timer_interrupt_status = InterruptStatus::Idle;
        }
    }
    
    pub fn read_tma(&self) -> u8 {
        self.tma
    }
    pub fn write_tma(&mut self, val: u8) {
        self.tma = val
    }
    
    pub fn read_tac(&self) -> u8 {
        serialize!(
            0b11111000;
            (self.tima_enabled as u8) =>> 2;
            (self.tima_speed.as_u2()) =>> 1..=0;
        )
    }
    
    pub fn write_tac(&mut self, val: u8) {
        deserialize!(val;
            2 as bool =>> tima_enabled;
            1..=0 =>> tima_speed;
        );
        let tima_speed = TimaSpeed::from_u2(tima_speed);

        let old_div_bit_high = self.div_full & self.tima_speed as u16 != 0;
        let new_div_bit_low = self.div_full & tima_speed as u16 == 0;
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
    
    pub fn read_key1(&self) -> u8 {
        serialize!(
            (self.system_speed.as_u1()) =>> 7;
            (self.speed_switch_queued as u8) =>> 0;
        )
    }

    pub fn write_key1(&mut self, val: u8) {
        deserialize!(val;
            0 as bool =>> (self.speed_switch_queued); 
        );
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

impl SystemSpeed {
    pub fn as_u1(&self) -> u8 {
        match self {
            Self::Standard => 0,
            Self::CgbDouble => 1,
        }
    }
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