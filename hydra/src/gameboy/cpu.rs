mod opcode;

use std::{cell::RefCell, rc::Rc, time::Duration};

use futures::FutureExt;

use crate::{
    common::{bit::BitVec, timing::{DelayedTickCounter, ModuloCounter}}, gameboy::{
        AGBRevision, CGBRevision, GBRevision, GameBoy, GbMode, Joypad, Model, SGBRevision, cpu::opcode::{CondOperand, ConstOperand16, IntOperand, OpcodeFn}, interrupt::{Interrupt, InterruptEnable, InterruptFlags}, memory::{
            MemoryMap, rom::Rom
        }, timer::MasterTimer
    },
};

pub enum Register8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

/// A Game Boy CPU.
///
/// Note: Registers are stored in little-endian byte arrays, so the representation in code may be misleading.
/// The AF register, for example, is indexed as follows:
/// ```
/// let af: u16 = u16::from_le_bytes(cpu.af);
/// let a: u8 = cpu.af[1];
/// let f: u8 = cpu.af[0];
/// let z: bool = (cpu.af[0] & 0b10000000) != 0;
/// let n: bool = (cpu.af[0] & 0b01000000) != 0;
/// let h: bool = (cpu.af[0] & 0b00100000) != 0;
/// let c: bool = (cpu.af[0] & 0b00010000) != 0;
///
/// cpu.af[0] &= 0b01111111; // Reset zero flag
/// cpu.af[0] |= 0b00010000; // Set carry flag
/// cpu.af[0] = ((true as u8) << 5) | (cpu.af[0] & 0b11011111) // Set/reset half-carry flag based on bool
/// ```
pub struct Cpu {
    mode: CpuMode,

    af: [u8; 2],
    bc: [u8; 2],
    de: [u8; 2],
    hl: [u8; 2],
    sp: u16,
    pc: u16,
    ir: u8,
    
    ime: bool,
    cycles_until_ime: Option<u8>,
    cycles_until_halt_bug: Option<u8>,
    unhalt_timer: DelayedTickCounter<u16>,
}

impl Cpu {
    pub fn new(rom: &Rom, model: &Rc<Model>, mode: &GbMode) -> Self {
        let dmg_mode = matches!(mode, GbMode::DMG);
        
        let af;
        let bc;
        let de;
        let hl;
        match **model {
            Model::GameBoy(GBRevision::DMG0) => {
                af = [0b0000 << 4, 0x01];
                bc = [0x13, 0xFF];
                de = [0xC1, 0x00];
                hl = [0x03, 0x84];
            }
            Model::GameBoy(GBRevision::DMG) => {
                af = [if rom.get_header_checksum() == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0x01];
                bc = [0x13, 0x00];
                de = [0xD8, 0x00];
                hl = [0x4D, 0x01];
            }
            Model::GameBoy(GBRevision::MGB) => {
                af = [if rom.get_header_checksum() == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0xFF];
                bc = [0x13, 0x00];
                de = [0xD8, 0x00];
                hl = [0x4D, 0x01];
            }
            Model::SuperGameBoy(SGBRevision::SGB) => {
                af = [0b0000 << 4, 0x01];
                bc = [0x14, 0x00];
                de = [0x00, 0x00];
                hl = [0x60, 0xC0];
            }
            Model::SuperGameBoy(SGBRevision::SGB2) => {
                af = [0b0000 << 4, 0xFF];
                bc = [0x14, 0x00];
                de = [0x00, 0x00];
                hl = [0x60, 0xC0];
            }
            Model::GameBoyColor(_) if dmg_mode => {
                let mut b = 0x00;
                let mut hl_bytes = [0x7C, 0x00];
                if rom.has_publisher_rnd1() {
                    // If either licensee code is 0x01, B = sum of all title bytes
                    b = rom.get_title().iter().sum();
                    if b == 0x43 || b == 0x58 {
                        // And, check special cases for HL
                        hl_bytes = [0x1A, 0x99];
                    }
                }
                af = [0b1000 << 4, 0x11];
                bc = [0x00, b];
                de = [0x08, 0x00];
                hl = hl_bytes;
            }
            Model::GameBoyAdvance(_) if dmg_mode => {
                let mut b = 0x01;
                let mut hl_bytes = [0x7C, 0x00];
                let mut f = 0b00000000;
                if rom.has_publisher_rnd1() {
                    // If either licensee code is 0x01, B = sum of all title bytes
                    b = rom.get_title().iter().sum();
                    if b & 0b1111 == 0 {
                        // Last op is an INC; set h flag...
                        f |= 0b0010 << 4;
                        if b == 0 {
                            // ...and z flag if necessary
                            f |= 0b1000 << 4
                        }
                    } else if b == 0x44 || b == 0x59 {
                        // Otherwise, still check special cases for HL
                        hl_bytes = [0x1A, 0x99];
                    }
                }
                af = [f, 0x11];
                bc = [0x00, b];
                de = [0x08, 0x00];
                hl = hl_bytes;
            }
            Model::GameBoyColor(_) => {
                af = [0b1000 << 4, 0x11];
                bc = [0x00, 0x00];
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            Model::GameBoyAdvance(_) => {
                af = [0b0000 << 4, 0x11];
                bc = [0x00, 0x01];
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            _ => panic!("Attempt to initialize Game Boy CPU without a proper revision"),
        }
        Cpu {
            mode: CpuMode::Normal,

            af,
            bc,
            de,
            hl,
            sp: 0xFFFE,
            pc: 0x0100,
            ir: 0x00,

            ime: false, 
            cycles_until_ime: None,
            cycles_until_halt_bug: None,
            unhalt_timer: DelayedTickCounter::new(None),
        }
    }

    pub fn queue_ime(&mut self) {
        self.cycles_until_ime = Some(2)
    }
    pub fn cancel_ime(&mut self) {
        self.cycles_until_ime = None
    }

    pub fn queue_halt_bug(&mut self) {
        self.cycles_until_halt_bug = Some(2)
    }
    
    pub fn refresh_interrupt_handler(&mut self, pc_old: u16) {
        let decrements_to_zero = |n: &mut u8| {*n -= 1; *n == 0};
        if let Some(_) = self.cycles_until_ime.take_if(decrements_to_zero) {self.ime = true;}
        if let Some(_) = self.cycles_until_halt_bug.take_if(decrements_to_zero) {self.pc = pc_old;}
    }

    #[inline(always)]
    pub fn interrupt_pending(&self, system: &mut GameBoy) -> bool {
        system.memory.interrupt_enable.read_ie() & system.memory.interrupt_flags.read_if() != 0
    }

    #[inline(always)]
    fn fetch(&mut self, system: &mut GameBoy, debug: bool) {
        if self.ime && self.interrupt_pending(system) { 
            // Generate call instruction from handled interrupt
            let composite = system.memory.interrupt_enable.read_ie() & system.memory.interrupt_flags.read_if();
            for shift_width in 0..=4 {
                let bitmask = 1 << shift_width;
                if composite & bitmask == 0 {
                    // Check next priority interrupt
                    continue;
                } else {
                    // Handle interrupt and return call instruction
                    self.ime = false;
                    system.memory.interrupt_flags.get_inner().reset_bits(bitmask);
                    let jump_addr = (shift_width * 0x8) + 0x40;
                    
                    // TODO: Verify cycle accuracy
                    self.call(system, CondOperand::Unconditional, ConstOperand16(jump_addr));
                    return;
                }
            }
            unreachable!() // Interrupt should've been handled
        } else {
            // Fetch instruction from memory
            self.ir = self.step_u8(system);
            if debug {
                println!(
                    "{:#06X}: {:02X}  ---  A: {:#04X}   F: {:08b}   BC: {:#06X}   DE: {:#06X}   HL: {:#06X}   SP: {:#06X}",
                    self.pc - 1,
                    self.ir,
                    self.af[1],
                    self.af[0],
                    u16::from_le_bytes(self.bc),
                    u16::from_le_bytes(self.de),
                    u16::from_le_bytes(self.hl),
                    self.sp
                );
            }
            Self::OP_TABLE[self.ir as usize](self, system);
        }
    }

    #[inline(always)]
    fn step_u8(&mut self, system: &mut GameBoy) -> u8 {
        let result = system.memory.read_u8(self.pc, false);
        self.pc += 1;
        system.cycle_components();
        result
    }

    #[inline(always)]
    fn read_u8(&self, address: u16, system: &mut GameBoy) -> u8 {
        system.cycle_components();
        system.memory.read_u8(address, false)
    }

    #[inline(always)]
    fn write_u8(&self, address: u16, value: u8, system: &mut GameBoy) -> () {
        system.cycle_components();
        system.memory.write_u8(value, address, false);
    }

    pub fn coro(&mut self, system: &mut GameBoy, debug: bool) {
        while system.is_running() {
            // Skip iterations if halted or stopped
            self.mode = match self.mode {
                CpuMode::Normal => CpuMode::Normal,
                CpuMode::Halted if self.interrupt_pending(system) || self.unhalt_timer.increment() => CpuMode::Normal,
                CpuMode::Stopped if system.memory.interrupt_flags.is_requested(Interrupt::Joypad) => CpuMode::Normal,
                _ => {
                    system.cycle_components();
                    continue;  
                }
            };

            // Fetch cycle
            let pc_old = self.pc;
            self.fetch(system, debug);

            // Execute cycle(s)
            self.refresh_interrupt_handler(pc_old);
        }
    }
}

// Opcode Helpers
macro_rules! _offset {
    (z) => {
        7
    };
    (n) => {
        6
    };
    (h) => {
        5
    };
    (c) => {
        4
    };
}
macro_rules! _inverse_mask {
    (z) => {
        0b01111111
    };
    (n) => {
        0b10111111
    };
    (h) => {
        0b11011111
    };
    (c) => {
        0b11101111
    };
}
macro_rules! set_flags {
    ($cpu:expr; $($key:ident=$val:expr),* $(,)?) => {
        $(
            $cpu.af[0] = ($cpu.af[0] & _inverse_mask!($key)) | (($val as u8) << _offset!($key));
        )*
    };
}
macro_rules! _mask {
    (z) => {
        0b10000000
    };
    (n) => {
        0b01000000
    };
    (h) => {
        0b00100000
    };
    (c) => {
        0b00010000
    };
}
macro_rules! get_flag {
    ($cpu:expr; $flag:ident) => {
        $cpu.af[0] & _mask!($flag) != 0
    };
}

macro_rules! reg {
    ($self:ident.A) => { $self.af[1] };
    ($self:ident.F) => { $self.af[0] };
    ($self:ident.B) => { $self.bc[1] };
    ($self:ident.C) => { $self.bc[0] };
    ($self:ident.D) => { $self.de[1] };
    ($self:ident.E) => { $self.de[0] };
    ($self:ident.H) => { $self.hl[1] };
    ($self:ident.L) => { $self.hl[0] };
}

impl Cpu {
    #[inline(always)]
    fn ld<T, O1: IntOperand<T>, O2: IntOperand<T>>(&mut self, system: &mut GameBoy, dest: O1, src: O2) {
        let value = src.get(self, system);
        dest.set(value, self, system);
    }
    #[inline(always)]
    fn ld_hlspe(&mut self, system: &mut GameBoy) {
        let e = self.step_u8(system);

        let [sp_lsb, sp_msb] = self.sp.to_le_bytes();
        let (sp_half_carry, e_half_carry, adjustment) = match e.test_bit(7) {
            false => (sp_lsb | 0xF0, e & 0x0F, 0x00),
            true => (sp_lsb & 0x0F, e | 0xF0, 0xFF),
        };
        let (result_l, carry) = (sp_lsb).overflowing_add(e);
        let (_, half_carry) = (sp_half_carry).overflowing_add(e_half_carry);
        reg!(self.L) = result_l;
        set_flags!(self;
            z=(false),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        system.cycle_components();

        let (result_h, _) = sp_msb.carrying_add(adjustment, carry);
        reg!(self.H) = result_h;
    }

    #[inline(always)]
    fn inc<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system);
        let (result, _) = o.overflowing_add(1);
        let (_, half_carry) = (o | 0xF0).overflowing_add(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry)
        );
        operand.set(result, self, system);
    }
    #[inline(always)]
    fn inc16<O: IntOperand<u16>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system);
        let result = o.wrapping_add(1);
        system.cycle_components();
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn dec<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system);
        let (result, _) = o.overflowing_sub(1);
        let (_, half_carry) = (o & 0x0F).overflowing_sub(1);
        set_flags!(self;
            z=(result == 0),
            n=(true),
            h=(half_carry)
        );
        operand.set(result, self, system);
    }
    #[inline(always)]
    fn dec16<O: IntOperand<u16>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system);
        let result = o.wrapping_sub(1);
        system.cycle_components();
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn add<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let (a, operand) = (reg!(self.A), operand.get(self, system));
        let (result, carry) = a.overflowing_add(operand);
        let (_, half_carry) = (a | 0xF0).overflowing_add(operand & 0x0F);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        reg!(self.A) = result;
    }
    #[inline(always)]
    fn add_hl<O: IntOperand<u16>>(&mut self, system: &mut GameBoy, operand: O) {
        let (hl, operand) = (u16::from_le_bytes(self.hl), operand.get(self, system));
        let (result, carry) = hl.overflowing_add(operand);
        let (_, half_carry) = (hl | 0xF000).overflowing_add(operand & 0x0FFF);
        set_flags!(self;
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.hl = u16::to_le_bytes(result);
    }
    #[inline(always)]
    fn add_spe(&mut self, system: &mut GameBoy) {
        let e = self.step_u8(system);

        let [sp_lsb, sp_msb] = self.sp.to_le_bytes();
        let (sp_half_carry, e_half_carry, adjustment) = match e.test_bit(7) {
            false => (sp_lsb | 0xF0, e & 0x0F, 0x00),
            true => (sp_lsb & 0x0F, e | 0xF0, 0xFF),
        };
        let (result_l, carry) = (sp_lsb).overflowing_add(e);
        let (_, half_carry) = (sp_half_carry).overflowing_add(e_half_carry);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        system.cycle_components();

        let (result_h, _) = sp_msb.carrying_add(adjustment, carry);
        system.cycle_components();

        self.sp = u16::from_le_bytes([result_l, result_h]);
    }

    #[inline(always)]
    fn adc<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let (a, operand) = (reg!(self.A), operand.get(self, system));
        let (result, carry) = a.carrying_add(operand, get_flag!(self; c));
        let (_, half_carry) = (a | 0xF0).carrying_add(operand & 0x0F, get_flag!(self; c));
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        reg!(self.A) = result;
    }

    #[inline(always)]
    fn sub<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let (a, operand) = (reg!(self.A), operand.get(self, system));
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand & 0x0F);
        set_flags!(self;
            z=(result == 0),
            n=(true),
            h=(half_carry),
            c=(carry)
        );
        reg!(self.A) = result;
    }

    #[inline(always)]
    fn sbc<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let (a, operand) = (reg!(self.A), operand.get(self, system));
        let (result, carry1) = a.overflowing_sub(operand);
        let (result, carry2) = result.overflowing_sub(get_flag!(self; c) as u8);
        let (hc_result, half_carry1) = (a & 0x0F).overflowing_sub(operand & 0x0F);
        let (_, half_carry2) = hc_result.overflowing_sub(get_flag!(self; c) as u8);
        set_flags!(self;
            z=(result == 0),
            n=(true),
            h=(half_carry1 | half_carry2),
            c=(carry1 | carry2)
        );
        reg!(self.A) = result;
    }

    #[inline(always)]
    fn and<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let result = reg!(self.A) & operand.get(self, system);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(true),
            c=(false)
        );
        reg!(self.A) = result;
    }

    #[inline(always)]
    fn or<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let result = reg!(self.A) | operand.get(self, system);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        reg!(self.A) = result;
    }

    #[inline(always)]
    fn xor<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let result = reg!(self.A) ^ operand.get(self, system);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        reg!(self.A) = result;
    }

    #[inline(always)]
    fn cp<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let (a, operand) = (reg!(self.A), operand.get(self, system));
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand & 0x0F);
        set_flags!(self;
            z=(result == 0),
            n=(true),
            h=(half_carry),
            c=(carry)
        );
    }

    #[inline(always)]
    fn push<O: IntOperand<u16>>(&mut self, system: &mut GameBoy, operand: O) {
        let bytes = u16::to_le_bytes(operand.get(self, system));
        system.cycle_components();
        self.sp -= 1;
        self.write_u8(self.sp, bytes[1], system);
        self.sp -= 1;
        self.write_u8(self.sp, bytes[0], system);
    }

    #[inline(always)]
    fn pop<O: IntOperand<u16>>(&mut self, system: &mut GameBoy, operand: O) {
        let mut bytes = [0; 2];
        bytes[0] = self.read_u8(self.sp, system);
        self.sp += 1;
        bytes[1] = self.read_u8(self.sp, system);
        self.sp += 1;
        operand.set(u16::from_le_bytes(bytes), self, system);
    }

    #[inline(always)]
    fn ccf(&mut self, system: &mut GameBoy) {
        set_flags!(self;
            n=(false),
            h=(false)
        );
        reg!(self.F) ^= _mask!(c);
    }

    #[inline(always)]
    fn scf(&mut self, system: &mut GameBoy) {
        set_flags!(self;
            n=(false),
            h=(false),
            c=(true)
        );
    }

    #[inline(always)]
    fn daa(&mut self, system: &mut GameBoy) {
        let a = reg!(self.A);
        let n = get_flag!(self; n);
        let h = get_flag!(self; h);
        let c = get_flag!(self; c);

        let (result, carry) = if n {
            let adjustment = (h as u8 * 0x06) + (c as u8 * 0x60);
            (a.wrapping_sub(adjustment), false)
        } else {
            let adjustment = ((c || a > 0x99) as u8 * 0x60) + ((h || a & 0x0F > 0x09) as u8 * 0x06);
            a.overflowing_add(adjustment)
        };

        set_flags!(self;
            z=(result == 0),
            h=(false),
            c=(c || carry),
        );

        reg!(self.A) = result;
    }

    #[inline(always)]
    fn cpl(&mut self, system: &mut GameBoy) {
        reg!(self.A) ^= 0xFF;
        set_flags!(self;
            n=(true),
            h=(true)
        );
    }

    #[inline(always)]
    fn rlc<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O, sets_z: bool) {
        let o = operand.get(self, system);
        let result = o.rotate_left(1);
        set_flags!(self;
            z=(sets_z && result == 0),
            n=(false),
            h=(false),
            c=(result & 0b00000001 != 0)
        );
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn rrc<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O, sets_z: bool) {
        let o = operand.get(self, system);
        let result = o.rotate_right(1);
        set_flags!(self;
            z=(sets_z && result == 0),
            n=(false),
            h=(false),
            c=(result & 0b10000000 != 0)
        );
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn rl<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O, sets_z: bool) {
        let o = operand.get(self, system);
        let carry = o.test_bit(7);
        let result = (o << 1) | get_flag!(self; c) as u8;
        set_flags!(self;
            z=(sets_z && result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn rr<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O, sets_z: bool) {
        let o = operand.get(self, system);
        let carry = o.test_bit(0);
        let result = (o >> 1) | ((get_flag!(self; c) as u8) << 7);
        set_flags!(self;
            z=(sets_z && result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn sla<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system);
        let carry = o.test_bit(7);
        let result = o << 1;
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn sra<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system) as i8;
        let carry = o.test_bit(0);
        let result = o >> 1;
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result as u8, self, system);
    }

    #[inline(always)]
    fn swap<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system);
        let result = (o & 0x0F) << 4 | (o & 0xF0) >> 4;
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn srl<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, operand: O) {
        let o = operand.get(self, system);
        let carry = o.test_bit(0);
        let result = o >> 1;
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result, self, system);
    }

    #[inline(always)]
    fn bit<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, index: u8, operand: O) {
        let o = operand.get(self, system);
        set_flags!(self;
            z=(o & (1 << index) == 0),
            n=(false),
            h=(true),
        );
    }

    #[inline(always)]
    fn res<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, index: u8, operand: O) {
        let o = operand.get(self, system);
        operand.set(o & ((1 << index) ^ 0b11111111), self, system);
    }

    #[inline(always)]
    fn set<O: IntOperand<u8>>(&mut self, system: &mut GameBoy, index: u8, operand: O) {
        let o = operand.get(self, system);
        operand.set(o | (1 << index), self, system);
    }

    #[inline(always)]
    fn jp<O: IntOperand<u16>>(&mut self, system: &mut GameBoy, condition: CondOperand, operand: O) {
        let addr = operand.get(self, system);
        if condition.evaluate(self) {
            system.cycle_components();
            self.pc = addr;
        }
    }

    #[inline(always)]
    fn jr<O: IntOperand<i8>>(&mut self, system: &mut GameBoy, condition: CondOperand, operand: O) {
        let e = operand.get(self, system) as i8;
        let addr = self.pc.wrapping_add_signed(e.into());
        if condition.evaluate(self) {
            system.cycle_components();
            self.pc = addr;
        }
    }

    #[inline(always)]
    fn call<O: IntOperand<u16>>(&mut self, system: &mut GameBoy, condition: CondOperand, operand: O) {
        let addr = operand.get(self, system);
        if condition.evaluate(self) {
            self.push(system, opcode::RegisterOperand16(Register16::PC));
            self.pc = addr;
        }
    }

    #[inline(always)]
    fn ret(&mut self, system: &mut GameBoy, condition: CondOperand) {
        system.cycle_components();
        if condition.evaluate(self) {
            self.pop(system, opcode::RegisterOperand16(Register16::PC));
            system.cycle_components();
        }
    }

    #[inline(always)]
    fn reti(&mut self, system: &mut GameBoy) {
        self.ret(system, CondOperand::Unconditional);
        self.ime = true;
    }

    #[inline(always)]
    fn ei(&mut self, system: &mut GameBoy) {
        self.queue_ime();
    }

    #[inline(always)]
    fn di(&mut self, system: &mut GameBoy) {
        self.ime = false;
        self.cancel_ime();
    }

    #[inline(always)]
    fn halt(&mut self, system: &mut GameBoy) {
        self.mode = CpuMode::Halted;

        // If IME is off but an interrupt is already pending, trigger halt bug
        if !self.ime && self.interrupt_pending(system) {
            self.queue_halt_bug();
        }
    }

    #[inline(always)]
    fn stop(&mut self, system: &mut GameBoy) {
        let (mode, speed_switch, extra_cycle, reset_div) = match (system.memory.joypad.is_input_active(), self.interrupt_pending(system), system.memory.timer.is_speed_switch_requested(), self.ime) {
            (true, true, _, _) => (CpuMode::Normal, false, false, false),
            (true, false, _, _) => (CpuMode::Halted, false, true, false),
            (false, true, false, _) => (CpuMode::Stopped, false, false, true),
            (false, false, false, _) => (CpuMode::Stopped, false, true, true),
            (false, false, true, _) => {self.unhalt_timer.count_to(0xFFFF); (CpuMode::Halted, true, true, true)},
            (false, true, true, true) => (CpuMode::Normal, true, false, true), // TODO: Change delay to be 0x20000 master clock cycles
            (false, true, true, false) => panic!("Triggered undefined STOP opcode behavior"),
        };

        if extra_cycle {
            let _ = self.step_u8(system);
        }

        self.mode = mode;
        
        if speed_switch {
            system.memory.timer.toggle_speed(&mut system.memory.apu_state);
        }
        
        if reset_div {
            system.memory.timer.write_div(&mut system.memory.apu_state);
        }
    }
}

#[derive(Debug)]
enum CpuMode {
    Normal,
    Halted,
    Stopped,
}