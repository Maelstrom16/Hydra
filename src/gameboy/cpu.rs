mod opcode;

use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, Barrier, MutexGuard, RwLock},
    thread::{self, JoinHandle, ScopedJoinHandle, Thread},
};

use futures::lock::Mutex;

use crate::gameboy::{
    AGBRevision, CGBRevision, GBRevision, GameBoy, Model, SGBRevision,
    cpu::{
        self,
        opcode::{CondOperand, IntOperand, RegisterOperand8},
    },
    memory::{
        self, Memory, TITLE_ADDRESS,
        io::{self, IO},
    },
};

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
pub struct CPU {
    af: [u8; 2],
    bc: [u8; 2],
    de: [u8; 2],
    hl: [u8; 2],
    sp: u16,
    pc: u16,
    ir: u8,
    pub ie: Rc<Cell<u8>>,
    ime: bool,
    ime_queued: bool,
}

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

impl CPU {
    pub fn new(rom: &Box<[u8]>, io: &IO, model: Model) -> Self {
        let af;
        let bc;
        let de;
        let hl;
        const sp: u16 = 0xFFFE;
        const pc: u16 = 0x0100;
        const ir: u8 = 0x00;
        let ie: Rc<Cell<u8>> = io[io::IE].clone();
        const ime: bool = false;
        const ime_queued: bool = false;
        match model {
            Model::GameBoy(Some(GBRevision::DMG0)) => {
                af = [0b0000 << 4, 0x01];
                bc = [0x13, 0xFF];
                de = [0xC1, 0x00];
                hl = [0x03, 0x84];
            }
            Model::GameBoy(Some(GBRevision::DMG)) => {
                af = [if rom[memory::HEADER_CHECKSUM_ADDRESS] == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0x01];
                bc = [0x13, 0x00];
                de = [0xD8, 0x00];
                hl = [0x4D, 0x01];
            }
            Model::GameBoy(Some(GBRevision::MGB)) => {
                af = [if rom[memory::HEADER_CHECKSUM_ADDRESS] == 0 { 0b1000 << 4 } else { 0b1011 << 4 }, 0xFF];
                bc = [0x13, 0x00];
                de = [0xD8, 0x00];
                hl = [0x4D, 0x01];
            }
            Model::SuperGameBoy(Some(SGBRevision::SGB)) => {
                af = [0b0000 << 4, 0x01];
                bc = [0x14, 0x00];
                de = [0x00, 0x00];
                hl = [0x60, 0xC0];
            }
            Model::SuperGameBoy(Some(SGBRevision::SGB2)) => {
                af = [0b0000 << 4, 0xFF];
                bc = [0x14, 0x00];
                de = [0x00, 0x00];
                hl = [0x60, 0xC0];
            }
            Model::GameBoy(Some(GBRevision::CGBdmg)) => {
                let mut b = 0x00;
                let mut hl_bytes = [0x7C, 0x00];
                if rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x01 || rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x33 && rom[memory::NEW_LICENSEE_CODE_ADDRESS] == 0x01 {
                    for offset in 0..16 {
                        // If either licensee code is 0x01, B = sum of all title bytes
                        b += rom[TITLE_ADDRESS + offset];
                    }
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
            Model::GameBoy(Some(GBRevision::AGBdmg)) => {
                let mut b = 0x01;
                let mut hl_bytes = [0x7C, 0x00];
                let mut f = 0b00000000;
                if rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x01 || rom[memory::OLD_LICENSEE_CODE_ADDRESS] == 0x33 && rom[memory::NEW_LICENSEE_CODE_ADDRESS] == 0x01 {
                    for offset in 0..16 {
                        // If either licensee code is 0x01, B = sum of all title bytes
                        b += rom[TITLE_ADDRESS + offset];
                    }
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
            Model::GameBoyColor(Some(CGBRevision::CGB0 | CGBRevision::CGB)) => {
                af = [0b1000 << 4, 0x11];
                bc = [0x00, 0x00];
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            Model::GameBoyAdvance(Some(AGBRevision::AGB0 | AGBRevision::AGB)) => {
                af = [0b0000 << 4, 0x11];
                bc = [0x00, 0x01];
                de = [0x56, 0xFF];
                hl = [0x0D, 0x00];
            }
            _ => panic!("Attempt to initialize Game Boy CPU without a proper revision"),
        }
        CPU {
            af,
            bc,
            de,
            hl,
            sp,
            pc,
            ir,
            ie,
            ime,
            ime_queued,
        }
    }

    #[inline(always)]
    fn step_u8_and_wait(&mut self, memory: &Memory) -> u8 {
        let result = memory.read_u8(self.pc);
        self.pc += 1;

        result
    }

    #[inline(always)]
    fn read_u8_and_wait(&self, memory: &Memory, address: u16) -> u8 {
        memory.read_u8(address)
    }

    #[inline(always)]
    fn write_u8_and_wait(&self, memory: &mut Memory, address: u16, value: u8) -> () {
        memory.write_u8(value, address);
    }

    pub fn step(&mut self, memory: &mut Memory) {
        // Fetch cycle
        // print!("{:#06X}: ", self.pc);
        self.ir = self.step_u8_and_wait(memory);
        // println!("{:02X}  ---  A: {:#04X}   F: {:08b}   BC: {:#06X}   DE: {:#06X}   HL: {:#06X}   SP: {:#06X}", self.ir, self.af[1], self.af[0], u16::from_le_bytes(self.bc), u16::from_le_bytes(self.de), u16::from_le_bytes(self.hl), self.sp);
        if self.ime_queued {
            self.ime = true;
            self.ime_queued = false;
        }

        // Execute cycle(s)
        opcode::OPCODE_FUNCTIONS[self.ir as usize](self, memory);
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

impl CPU {
    #[inline(always)]
    fn ld<T, O1: IntOperand<T>, O2: IntOperand<T>>(&mut self, memory: &mut Memory, dest: O1, src: O2) {
        dest.set(src.get(self, memory), self, memory);
    }
    #[inline(always)]
    fn ld_hlspe(&mut self, memory: &mut Memory) {
        let e = self.step_u8_and_wait(memory) as i8;
        let result = self.sp.wrapping_add_signed(e.into());
        let lsb = (self.sp & 0xFF) as u8;
        let (_, carry) = lsb.overflowing_add_signed(e);
        let lsb_half = if e.signum() == 1 { lsb | 0xF0 } else { lsb & 0x0F };
        let (_, half_carry) = lsb_half.overflowing_add_signed(e);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(half_carry),
            c=(carry)
        );

        self.hl = u16::to_le_bytes(result);
    }

    #[inline(always)]
    fn inc<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let (result, _) = o.overflowing_add(1);
        let (_, half_carry) = (o | 0xF0).overflowing_add(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry)
        );
        operand.set(result, self, memory);
    }
    #[inline(always)]
    fn inc16<O: IntOperand<u16>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let result = o.wrapping_add(1);

        operand.set(result, self, memory);
    }

    #[inline(always)]
    fn dec<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let (result, _) = o.overflowing_sub(1);
        let (_, half_carry) = (o & 0x0F).overflowing_sub(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry)
        );
        operand.set(result, self, memory);
    }
    #[inline(always)]
    fn dec16<O: IntOperand<u16>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let result = o.wrapping_sub(1);

        operand.set(result, self, memory);
    }

    #[inline(always)]
    fn add<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let (a, operand) = (self.af[1], operand.get(self, memory));
        let (result, carry) = a.overflowing_add(operand);
        let (_, half_carry) = (a | 0xF0).overflowing_add(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }
    #[inline(always)]
    fn add_hl<O: IntOperand<u16>>(&mut self, memory: &mut Memory, operand: O) {
        let (hl, operand) = (u16::from_le_bytes(self.hl), operand.get(self, memory));
        let result = hl.wrapping_add(operand);
        let [_, h] = self.hl;
        let [_, oph] = u16::to_le_bytes(operand);
        let (_, carry) = h.overflowing_add(oph);
        let (_, half_carry) = (h | 0xF0).overflowing_add(oph);
        set_flags!(self;
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.hl = u16::to_le_bytes(result);
    }
    #[inline(always)]
    fn add_spe(&mut self, memory: &mut Memory) {
        let e = self.step_u8_and_wait(memory) as i8;
        let result = self.sp.wrapping_add_signed(e.into());
        let lsb = (self.sp & 0xFF) as u8;
        let (_, carry) = lsb.overflowing_add_signed(e);
        let lsb_half = if e.signum() == 1 { lsb | 0xF0 } else { lsb & 0x0F };
        let (_, half_carry) = lsb_half.overflowing_add_signed(e);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(half_carry),
            c=(carry)
        );

        self.sp = result;
    }

    #[inline(always)]
    fn adc<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let (a, operand) = (self.af[1], operand.get(self, memory) + get_flag!(self; c) as u8);
        let (result, carry) = a.overflowing_add(operand);
        let (_, half_carry) = (a | 0xF0).overflowing_add(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    fn sub<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let (a, operand) = (self.af[1], operand.get(self, memory));
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    fn sbc<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let (a, operand) = (self.af[1], operand.get(self, memory) + get_flag!(self; c) as u8);
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    fn and<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let result = self.af[1] & operand.get(self, memory);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(true),
            c=(false)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    fn or<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let result = self.af[1] | operand.get(self, memory);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    fn xor<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let result = self.af[1] ^ operand.get(self, memory);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        self.af[1] = result;
    }

    #[inline(always)]
    fn cp<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let (a, operand) = (self.af[1], operand.get(self, memory));
        let (result, carry) = a.overflowing_sub(operand);
        let (_, half_carry) = (a & 0x0F).overflowing_sub(operand);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(half_carry),
            c=(carry)
        );
    }

    #[inline(always)]
    fn push<O: IntOperand<u16>>(&mut self, memory: &mut Memory, operand: O) {
        let bytes = u16::to_le_bytes(operand.get(self, memory));

        self.sp -= 1;
        self.write_u8_and_wait(memory, self.sp, bytes[1]);
        self.sp -= 1;
        self.write_u8_and_wait(memory, self.sp, bytes[0]);
    }

    #[inline(always)]
    fn pop<O: IntOperand<u16>>(&mut self, memory: &mut Memory, operand: O) {
        let mut bytes = [0; 2];
        bytes[0] = self.read_u8_and_wait(memory, self.sp);
        self.sp += 1;
        bytes[1] = self.read_u8_and_wait(memory, self.sp);
        self.sp += 1;
        operand.set(u16::from_le_bytes(bytes), self, memory);
    }

    #[inline(always)]
    fn ccf(&mut self, memory: &mut Memory) {
        set_flags!(self;
            n=(false),
            h=(false)
        );
        self.af[0] ^= _mask!(c);
    }

    #[inline(always)]
    fn scf(&mut self, memory: &mut Memory) {
        set_flags!(self;
            n=(false),
            h=(false),
            c=(true)
        );
    }

    #[inline(always)]
    fn daa(&mut self, memory: &mut Memory) {
        todo!() //TODO
    }

    #[inline(always)]
    fn cpl(&mut self, memory: &mut Memory) {
        self.af[1] ^= 0xFF;
        set_flags!(self;
            n=(true),
            h=(true)
        );
    }

    #[inline(always)]
    fn rlc<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let result = o.rotate_left(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(result & 0b00000001 != 0)
        );
        operand.set(result, self, memory);
    }

    #[inline(always)]
    fn rrc<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let result = o.rotate_right(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(result & 0b10000000 != 0)
        );
        operand.set(result, self, memory);
    }

    #[inline(always)]
    fn rl<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let (result, carry) = o.overflowing_shl(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result | get_flag!(self; c) as u8, self, memory);
    }

    #[inline(always)]
    fn rr<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let (result, carry) = o.overflowing_shr(1);
        set_flags!(self;
            z=(false),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result | (get_flag!(self; c) as u8) << 7, self, memory);
    }

    #[inline(always)]
    fn sla<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let (result, carry) = o.overflowing_shl(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result, self, memory);
    }

    #[inline(always)]
    fn sra<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory) as i8;
        let (result, carry) = o.overflowing_shr(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result as u8, self, memory);
    }

    #[inline(always)]
    fn swap<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let result = (o & 0x0F) << 4 | (o & 0xF0) >> 4;
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(false)
        );
        operand.set(result, self, memory);
    }

    #[inline(always)]
    fn srl<O: IntOperand<u8>>(&mut self, memory: &mut Memory, operand: O) {
        let o = operand.get(self, memory);
        let (result, carry) = o.overflowing_shr(1);
        set_flags!(self;
            z=(result == 0),
            n=(false),
            h=(false),
            c=(carry)
        );
        operand.set(result, self, memory);
    }

    #[inline(always)]
    fn bit<O: IntOperand<u8>>(&mut self, memory: &mut Memory, index: u8, operand: O) {
        let o = operand.get(self, memory);
        set_flags!(self;
            z=(o & (1 << index) != 0),
            n=(false),
            h=(true),
        );
    }

    #[inline(always)]
    fn res<O: IntOperand<u8>>(&mut self, memory: &mut Memory, index: u8, operand: O) {
        let o = operand.get(self, memory);
        operand.set(o & ((1 << index) ^ 0b11111111), self, memory);
    }

    #[inline(always)]
    fn set<O: IntOperand<u8>>(&mut self, memory: &mut Memory, index: u8, operand: O) {
        let o = operand.get(self, memory);
        operand.set(o | (1 << index), self, memory);
    }

    #[inline(always)]
    fn jp<O: IntOperand<u16>>(&mut self, memory: &mut Memory, condition: CondOperand, operand: O) {
        let addr = operand.get(self, memory);
        if condition.evaluate(self) {
            self.pc = addr;
        }
    }

    #[inline(always)]
    fn jr<O: IntOperand<i8>>(&mut self, memory: &mut Memory, condition: CondOperand, operand: O) {
        let e = operand.get(self, memory) as i8;
        let addr = self.pc.wrapping_add_signed(e.into());
        if condition.evaluate(self) {
            self.pc = addr;
        }
    }

    #[inline(always)]
    fn call<O: IntOperand<u16>>(&mut self, memory: &mut Memory, condition: CondOperand, operand: O) {
        let addr = operand.get(self, memory);
        if condition.evaluate(self) {
            self.push(memory, opcode::RegisterOperand16(Register16::PC));
            self.pc = addr;
        }
    }

    #[inline(always)]
    fn ret(&mut self, memory: &mut Memory, condition: CondOperand) {
        if condition.evaluate(self) {
            self.pop(memory, opcode::RegisterOperand16(Register16::PC));
        }
    }

    #[inline(always)]
    fn reti(&mut self, memory: &mut Memory) {
        self.ret(memory, CondOperand::Unconditional);
        self.ime = true;
    }

    #[inline(always)]
    fn ei(&mut self, memory: &mut Memory) {
        self.ime_queued = true;
    }

    #[inline(always)]
    fn di(&mut self, memory: &mut Memory) {
        self.ime = false;
    }

    #[inline(always)]
    fn halt(&mut self, memory: &mut Memory) {
        todo!() //TODO
    }

    #[inline(always)]
    fn stop(&mut self, memory: &mut Memory) {
        todo!() //TODO
    }
}
