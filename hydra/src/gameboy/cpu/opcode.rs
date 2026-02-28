use std::{cell::RefCell, pin::Pin, rc::Rc};

use futures::FutureExt;

use crate::{
    gameboy::{GameBoy, GbMode, cpu::{self, Cpu}},
};

pub trait IntOperand<T> {
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> T;
    fn set(&self, value: T, cpu: &mut Cpu, system: &mut GameBoy);
}

pub struct RegisterOperand8(pub cpu::Register8);
impl IntOperand<u8> for RegisterOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, _system: &mut GameBoy) -> u8 {
        match self.0 {
            cpu::Register8::A => cpu.af[1],
            cpu::Register8::F => cpu.af[0],
            cpu::Register8::B => cpu.bc[1],
            cpu::Register8::C => cpu.bc[0],
            cpu::Register8::D => cpu.de[1],
            cpu::Register8::E => cpu.de[0],
            cpu::Register8::H => cpu.hl[1],
            cpu::Register8::L => cpu.hl[0],
        }
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut Cpu, _system: &mut GameBoy) {
        match self.0 {
            cpu::Register8::A => cpu.af[1] = value,
            cpu::Register8::F => cpu.af[0] = value & 0xF0,
            cpu::Register8::B => cpu.bc[1] = value,
            cpu::Register8::C => cpu.bc[0] = value,
            cpu::Register8::D => cpu.de[1] = value,
            cpu::Register8::E => cpu.de[0] = value,
            cpu::Register8::H => cpu.hl[1] = value,
            cpu::Register8::L => cpu.hl[0] = value,
        };
    }
}

pub struct ImmediateOperand8;
impl IntOperand<u8> for ImmediateOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u8 {
        cpu.step_u8(system)
    }
    #[inline(always)]
    fn set(&self, _: u8, _: &mut Cpu, _system: &mut GameBoy) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ImmediateSignedOperand8;
impl IntOperand<i8> for ImmediateSignedOperand8 {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> i8 {
        cpu.step_u8(system) as i8
    }
    #[inline(always)]
    fn set(&self, _: i8, _: &mut Cpu, _system: &mut GameBoy) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct IndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u8 {
        let address = self.0.get(cpu, system);
        cpu.read_u8(address, system)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut Cpu, system: &mut GameBoy) {
        let address = self.0.get(cpu, system);
        cpu.write_u8(address, value, system);
    }
}
pub struct IncIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for IncIndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u8 {
        let address = self.0.get(cpu, system);
        self.0.set(address + 1, cpu, system);
        cpu.read_u8(address, system)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut Cpu, system: &mut GameBoy) {
        let address = self.0.get(cpu, system);
        self.0.set(address + 1, cpu, system);
        cpu.write_u8(address, value, system);
    }
}
pub struct DecIndirectOperand8<O: IntOperand<u16>>(pub O);
impl<O: IntOperand<u16>> IntOperand<u8> for DecIndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u8 {
        let address = self.0.get(cpu, system);
        self.0.set(address - 1, cpu, system);
        cpu.read_u8(address, system)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut Cpu, system: &mut GameBoy) {
        let address = self.0.get(cpu, system);
        self.0.set(address - 1, cpu, system);
        cpu.write_u8(address, value, system);
    }
}

pub struct HramIndirectOperand<O: IntOperand<u8>>(pub O);
impl<O: IntOperand<u8>> HramIndirectOperand<O> {
    #[inline(always)]
    fn as_hram_address(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u16 {
        0xFF00 | (self.0.get(cpu, system)) as u16
    }
}
impl<O: IntOperand<u8>> IntOperand<u8> for HramIndirectOperand<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u8 {
        let hram_address = self.as_hram_address(cpu, system);
        cpu.read_u8(hram_address, system)
    }
    #[inline(always)]
    fn set(&self, value: u8, cpu: &mut Cpu, system: &mut GameBoy) {
        let hram_address = self.as_hram_address(cpu, system);
        cpu.write_u8(hram_address, value, system);
    }
}
impl<O: IntOperand<u16>> IntOperand<u16> for IndirectOperand8<O> {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u16 {
        let address = self.0.get(cpu, system);
        u16::from_le_bytes([
            cpu.read_u8(address, system),
            cpu.read_u8(address + 1, system),
        ])
    }
    #[inline(always)]
    fn set(&self, value: u16, cpu: &mut Cpu, system: &mut GameBoy) {
        let address = self.0.get(cpu, system);
        let bytes = u16::to_le_bytes(value);
        cpu.write_u8(address, bytes[0], system);
        cpu.write_u8(address + 1, bytes[1], system);
    }
}

pub struct RegisterOperand16(pub cpu::Register16);
impl IntOperand<u16> for RegisterOperand16 {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, _system: &mut GameBoy) -> u16 {
        match self.0 {
            cpu::Register16::AF => u16::from_le_bytes(cpu.af),
            cpu::Register16::BC => u16::from_le_bytes(cpu.bc),
            cpu::Register16::DE => u16::from_le_bytes(cpu.de),
            cpu::Register16::HL => u16::from_le_bytes(cpu.hl),
            cpu::Register16::SP => cpu.sp,
            cpu::Register16::PC => cpu.pc,
        }
    }
    #[inline(always)]
    fn set(&self, value: u16, cpu: &mut Cpu, _system: &mut GameBoy) {
        match self.0 {
            cpu::Register16::AF => cpu.af = u16::to_le_bytes(value & 0xFFF0),
            cpu::Register16::BC => cpu.bc = u16::to_le_bytes(value),
            cpu::Register16::DE => cpu.de = u16::to_le_bytes(value),
            cpu::Register16::HL => cpu.hl = u16::to_le_bytes(value),
            cpu::Register16::SP => cpu.sp = value,
            cpu::Register16::PC => cpu.pc = value,
        }
    }
}

pub struct ImmediateOperand16;
impl IntOperand<u16> for ImmediateOperand16 {
    #[inline(always)]
    fn get(&self, cpu: &mut Cpu, system: &mut GameBoy) -> u16 {
        u16::from_le_bytes([cpu.step_u8(system), cpu.step_u8(system)])
    }
    #[inline(always)]
    fn set(&self, _: u16, _: &mut Cpu, _system: &mut GameBoy) {
        panic!("Cannot write to immediate operand")
    }
}

pub struct ConstOperand16(pub u16);
impl IntOperand<u16> for ConstOperand16 {
    #[inline(always)]
    fn get(&self, _: &mut Cpu, _system: &mut GameBoy) -> u16 {
        self.0
    }
    #[inline(always)]
    fn set(&self, _: u16, _: &mut Cpu, _system: &mut GameBoy) {
        panic!("Cannot write to constant operand")
    }
}

pub enum CondOperand {
    Unconditional,
    NZ,
    Z,
    NC,
    C,
}
impl CondOperand {
    #[inline(always)]
    pub fn evaluate(&self, cpu: &Cpu) -> bool {
        match self {
            Self::Unconditional => true,
            Self::NZ => cpu.af[0] & 0b10000000 == 0,
            Self::Z => cpu.af[0] & 0b10000000 != 0,
            Self::NC => cpu.af[0] & 0b00010000 == 0,
            Self::C => cpu.af[0] & 0b00010000 != 0,
        }
    }
}


pub type OpcodeFn = fn(&mut Cpu, &mut GameBoy);

impl Cpu {
    pub(super) const OP_TABLE: [OpcodeFn; 0x100] = Self::generate_op();
    pub(super) const CB_TABLE: [OpcodeFn; 0x100] = Self::generate_cb();
    const INVALID: OpcodeFn = |_, _| {panic!("Unknown opcode")};

    const fn generate_op() -> [OpcodeFn; 0x100] {
        let mut op_table = [Self::INVALID; 0x100];
        op_table[0x00] = |cpu, system| { 
            // NOP
        };
        op_table[0x01] = |cpu, system| {
            // LD BC, nn
            cpu.ld(system, RegisterOperand16(cpu::Register16::BC), ImmediateOperand16);
        };
        op_table[0x02] = |cpu, system| {
            // LD (BC), A
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::BC)), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x03] = |cpu, system| {
            // INC BC
            cpu.inc16(system, RegisterOperand16(cpu::Register16::BC));
        };
        op_table[0x04] = |cpu, system| {
            // INC B
            cpu.inc(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x05] = |cpu, system| {
            // DEC B
            cpu.dec(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x06] = |cpu, system| {
            // LD B, n
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), ImmediateOperand8);
        };
        op_table[0x07] = |cpu, system| {
            // RLCA
            cpu.rlc(system, RegisterOperand8(cpu::Register8::A), false);
        };
        op_table[0x08] = |cpu, system| {
            // LD (nn), SP
            cpu.ld(system, IndirectOperand8(ImmediateOperand16), RegisterOperand16(cpu::Register16::SP));
        };
        op_table[0x09] = |cpu, system| {
            // ADD HL, BC
            cpu.add_hl(system, RegisterOperand16(cpu::Register16::BC));
        };
        op_table[0x0A] = |cpu, system| {
            // LD A, (BC)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::BC)));
        };
        op_table[0x0B] = |cpu, system| {
            // DEC BC
            cpu.dec16(system, RegisterOperand16(cpu::Register16::BC));
        };
        op_table[0x0C] = |cpu, system| {
            // INC C
            cpu.inc(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x0D] = |cpu, system| {
            // DEC C
            cpu.dec(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x0E] = |cpu, system| {
            // LD C, n
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), ImmediateOperand8);
        };
        op_table[0x0F] = |cpu, system| {
            // RRCA
            cpu.rrc(system, RegisterOperand8(cpu::Register8::A), false);
        };

        op_table[0x10] = |cpu, system| {
            // STOP
            cpu.stop(system);
        };
        op_table[0x11] = |cpu, system| {
            // LD DE, nn
            cpu.ld(system, RegisterOperand16(cpu::Register16::DE), ImmediateOperand16);
        };
        op_table[0x12] = |cpu, system| {
            // LD (DE), A
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::DE)), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x13] = |cpu, system| {
            // INC DE
            cpu.inc16(system, RegisterOperand16(cpu::Register16::DE));
        };
        op_table[0x14] = |cpu, system| {
            // INC D
            cpu.inc(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x15] = |cpu, system| {
            // DEC D
            cpu.dec(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x16] = |cpu, system| {
            // LD D, n
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), ImmediateOperand8);
        };
        op_table[0x17] = |cpu, system| {
            // RLA
            cpu.rl(system, RegisterOperand8(cpu::Register8::A), false);
        };
        op_table[0x18] = |cpu, system| {
            // JR e
            cpu.jr(system, CondOperand::Unconditional, ImmediateSignedOperand8);
        };
        op_table[0x19] = |cpu, system| {
            // ADD HL, DE
            cpu.add_hl(system, RegisterOperand16(cpu::Register16::DE));
        };
        op_table[0x1A] = |cpu, system| {
            // LD A, (DE)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::DE)));
        };
        op_table[0x1B] = |cpu, system| {
            // DEC DE
            cpu.dec16(system, RegisterOperand16(cpu::Register16::DE));
        };
        op_table[0x1C] = |cpu, system| {
            // INC E
            cpu.inc(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x1D] = |cpu, system| {
            // DEC E
            cpu.dec(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x1E] = |cpu, system| {
            // LD E, n
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), ImmediateOperand8);
        };
        op_table[0x1F] = |cpu, system| {
            // RRA
            cpu.rr(system, RegisterOperand8(cpu::Register8::A), false);
        };

        op_table[0x20] = |cpu, system| {
            // JR NZ, e
            cpu.jr(system, CondOperand::NZ, ImmediateSignedOperand8);
        };
        op_table[0x21] = |cpu, system| {
            // LD HL, nn
            cpu.ld(system, RegisterOperand16(cpu::Register16::HL), ImmediateOperand16);
        };
        op_table[0x22] = |cpu, system| {
            // LD (HL+), A
            cpu.ld(system, IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x23] = |cpu, system| {
            // INC HL
            cpu.inc16(system, RegisterOperand16(cpu::Register16::HL));
        };
        op_table[0x24] = |cpu, system| {
            // INC H
            cpu.inc(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x25] = |cpu, system| {
            // DEC H
            cpu.dec(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x26] = |cpu, system| {
            // LD H, n
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), ImmediateOperand8);
        };
        op_table[0x27] = |cpu, system| {
            // DAA
            cpu.daa(system);
        };
        op_table[0x28] = |cpu, system| {
            // JR Z, e
            cpu.jr(system, CondOperand::Z, ImmediateSignedOperand8);
        };
        op_table[0x29] = |cpu, system| {
            // ADD HL, HL
            cpu.add_hl(system, RegisterOperand16(cpu::Register16::HL));
        };
        op_table[0x2A] = |cpu, system| {
            // LD A, (HL+)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), IncIndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x2B] = |cpu, system| {
            // DEC HL
            cpu.dec16(system, RegisterOperand16(cpu::Register16::HL));
        };
        op_table[0x2C] = |cpu, system| {
            // INC L
            cpu.inc(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x2D] = |cpu, system| {
            // DEC L
            cpu.dec(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x2E] = |cpu, system| {
            // LD L, n
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), ImmediateOperand8);
        };
        op_table[0x2F] = |cpu, system| {
            // CPL
            cpu.cpl(system);
        };

        op_table[0x30] = |cpu, system| {
            // JR NC, e
            cpu.jr(system, CondOperand::NC, ImmediateSignedOperand8);
        };
        op_table[0x31] = |cpu, system| {
            // LD SP, nn
            cpu.ld(system, RegisterOperand16(cpu::Register16::SP), ImmediateOperand16);
        };
        op_table[0x32] = |cpu, system| {
            // LD (HL-), A
            cpu.ld(system, DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x33] = |cpu, system| {
            // INC SP
            cpu.inc16(system, RegisterOperand16(cpu::Register16::SP));
        };
        op_table[0x34] = |cpu, system| {
            // INC (HL)
            cpu.inc(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x35] = |cpu, system| {
            // DEC (HL)
            cpu.dec(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x36] = |cpu, system| {
            // LD (HL), n
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), ImmediateOperand8);
        };
        op_table[0x37] = |cpu, system| {
            // SCF
            cpu.scf(system);
        };
        op_table[0x38] = |cpu, system| {
            // JR C, e
            cpu.jr(system, CondOperand::C, ImmediateSignedOperand8);
        };
        op_table[0x39] = |cpu, system| {
            // ADD HL, SP
            cpu.add_hl(system, RegisterOperand16(cpu::Register16::SP));
        };
        op_table[0x3A] = |cpu, system| {
            // LD A, (HL-)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), DecIndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x3B] = |cpu, system| {
            // DEC SP
            cpu.dec16(system, RegisterOperand16(cpu::Register16::SP));
        };
        op_table[0x3C] = |cpu, system| {
            // INC A
            cpu.inc(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x3D] = |cpu, system| {
            // DEC A
            cpu.dec(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x3E] = |cpu, system| {
            // LD A, n
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), ImmediateOperand8);
        };
        op_table[0x3F] = |cpu, system| {
            // CCF
            cpu.ccf(system);
        };

        op_table[0x40] = |cpu, system| {
            // LD B, B
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x41] = |cpu, system| {
            // LD B, C
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x42] = |cpu, system| {
            // LD B, D
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x43] = |cpu, system| {
            // LD B, E
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x44] = |cpu, system| {
            // LD B, H
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x45] = |cpu, system| {
            // LD B, L
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x46] = |cpu, system| {
            // LD B, (HL)
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x47] = |cpu, system| {
            // LD B, A
            cpu.ld(system, RegisterOperand8(cpu::Register8::B), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x48] = |cpu, system| {
            // LD C, B
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x49] = |cpu, system| {
            // LD C, C
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x4A] = |cpu, system| {
            // LD C, D
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x4B] = |cpu, system| {
            // LD C, E
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x4C] = |cpu, system| {
            // LD C, H
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x4D] = |cpu, system| {
            // LD C, L
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x4E] = |cpu, system| {
            // LD C, (HL)
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x4F] = |cpu, system| {
            // LD C, A
            cpu.ld(system, RegisterOperand8(cpu::Register8::C), RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x50] = |cpu, system| {
            // LD D, B
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x51] = |cpu, system| {
            // LD D, C
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x52] = |cpu, system| {
            // LD D, D
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x53] = |cpu, system| {
            // LD D, E
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x54] = |cpu, system| {
            // LD D, H
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x55] = |cpu, system| {
            // LD D, L
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x56] = |cpu, system| {
            // LD D, (HL)
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x57] = |cpu, system| {
            // LD D, A
            cpu.ld(system, RegisterOperand8(cpu::Register8::D), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x58] = |cpu, system| {
            // LD E, B
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x59] = |cpu, system| {
            // LD E, C
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x5A] = |cpu, system| {
            // LD E, D
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x5B] = |cpu, system| {
            // LD E, E
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x5C] = |cpu, system| {
            // LD E, H
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x5D] = |cpu, system| {
            // LD E, L
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x5E] = |cpu, system| {
            // LD E, (HL)
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x5F] = |cpu, system| {
            // LD E, A
            cpu.ld(system, RegisterOperand8(cpu::Register8::E), RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x60] = |cpu, system| {
            // LD H, B
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x61] = |cpu, system| {
            // LD H, C
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x62] = |cpu, system| {
            // LD H, D
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x63] = |cpu, system| {
            // LD H, E
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x64] = |cpu, system| {
            // LD H, H
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x65] = |cpu, system| {
            // LD H, L
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x66] = |cpu, system| {
            // LD H, (HL)
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x67] = |cpu, system| {
            // LD H, A
            cpu.ld(system, RegisterOperand8(cpu::Register8::H), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x68] = |cpu, system| {
            // LD L, B
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x69] = |cpu, system| {
            // LD L, C
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x6A] = |cpu, system| {
            // LD L, D
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x6B] = |cpu, system| {
            // LD L, E
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x6C] = |cpu, system| {
            // LD L, H
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x6D] = |cpu, system| {
            // LD L, L
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x6E] = |cpu, system| {
            // LD L, (HL)
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x6F] = |cpu, system| {
            // LD L, A
            cpu.ld(system, RegisterOperand8(cpu::Register8::L), RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x70] = |cpu, system| {
            // LD (HL), B
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x71] = |cpu, system| {
            // LD (HL), C
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x72] = |cpu, system| {
            // LD (HL), D
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x73] = |cpu, system| {
            // LD (HL), E
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x74] = |cpu, system| {
            // LD (HL), H
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x75] = |cpu, system| {
            // LD (HL), L
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x76] = |cpu, system| {
            // HALT
            cpu.halt(system);
        };
        op_table[0x77] = |cpu, system| {
            // LD (HL), A
            cpu.ld(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x78] = |cpu, system| {
            // LD A, B
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x79] = |cpu, system| {
            // LD A, C
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x7A] = |cpu, system| {
            // LD A, D
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x7B] = |cpu, system| {
            // LD A, E
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x7C] = |cpu, system| {
            // LD A, H
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x7D] = |cpu, system| {
            // LD A, L
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x7E] = |cpu, system| {
            // LD A, (HL)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x7F] = |cpu, system| {
            // LD A, A
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x80] = |cpu, system| {
            // ADD B
            cpu.add(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x81] = |cpu, system| {
            // ADD C
            cpu.add(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x82] = |cpu, system| {
            // ADD D
            cpu.add(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x83] = |cpu, system| {
            // ADD E
            cpu.add(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x84] = |cpu, system| {
            // ADD H
            cpu.add(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x85] = |cpu, system| {
            // ADD L
            cpu.add(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x86] = |cpu, system| {
            // ADD (HL)
            cpu.add(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x87] = |cpu, system| {
            // ADD A
            cpu.add(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x88] = |cpu, system| {
            // ADC B
            cpu.adc(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x89] = |cpu, system| {
            // ADC C
            cpu.adc(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x8A] = |cpu, system| {
            // ADC D
            cpu.adc(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x8B] = |cpu, system| {
            // ADC E
            cpu.adc(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x8C] = |cpu, system| {
            // ADC H
            cpu.adc(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x8D] = |cpu, system| {
            // ADC L
            cpu.adc(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x8E] = |cpu, system| {
            // ADC (HL)
            cpu.adc(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x8F] = |cpu, system| {
            // ADC A
            cpu.adc(system, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x90] = |cpu, system| {
            // SUB B
            cpu.sub(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x91] = |cpu, system| {
            // SUB C
            cpu.sub(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x92] = |cpu, system| {
            // SUB D
            cpu.sub(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x93] = |cpu, system| {
            // SUB E
            cpu.sub(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x94] = |cpu, system| {
            // SUB H
            cpu.sub(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x95] = |cpu, system| {
            // SUB L
            cpu.sub(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x96] = |cpu, system| {
            // SUB (HL)
            cpu.sub(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x97] = |cpu, system| {
            // SUB A
            cpu.sub(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x98] = |cpu, system| {
            // SBC B
            cpu.sbc(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x99] = |cpu, system| {
            // SBC C
            cpu.sbc(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x9A] = |cpu, system| {
            // SBC D
            cpu.sbc(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x9B] = |cpu, system| {
            // SBC E
            cpu.sbc(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x9C] = |cpu, system| {
            // SBC H
            cpu.sbc(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x9D] = |cpu, system| {
            // SBC L
            cpu.sbc(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x9E] = |cpu, system| {
            // SBC (HL)
            cpu.sbc(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x9F] = |cpu, system| {
            // SBC A
            cpu.sbc(system, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xA0] = |cpu, system| {
            // AND B
            cpu.and(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xA1] = |cpu, system| {
            // AND C
            cpu.and(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xA2] = |cpu, system| {
            // AND D
            cpu.and(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xA3] = |cpu, system| {
            // AND E
            cpu.and(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xA4] = |cpu, system| {
            // AND H
            cpu.and(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xA5] = |cpu, system| {
            // AND L
            cpu.and(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xA6] = |cpu, system| {
            // AND (HL)
            cpu.and(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xA7] = |cpu, system| {
            // AND A
            cpu.and(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xA8] = |cpu, system| {
            // XOR B
            cpu.xor(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xA9] = |cpu, system| {
            // XOR C
            cpu.xor(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xAA] = |cpu, system| {
            // XOR D
            cpu.xor(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xAB] = |cpu, system| {
            // XOR E
            cpu.xor(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xAC] = |cpu, system| {
            // XOR H
            cpu.xor(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xAD] = |cpu, system| {
            // XOR L
            cpu.xor(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xAE] = |cpu, system| {
            // XOR (HL)
            cpu.xor(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xAF] = |cpu, system| {
            // XOR A
            cpu.xor(system, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xB0] = |cpu, system| {
            // OR B
            cpu.or(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xB1] = |cpu, system| {
            // OR C
            cpu.or(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xB2] = |cpu, system| {
            // OR D
            cpu.or(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xB3] = |cpu, system| {
            // OR E
            cpu.or(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xB4] = |cpu, system| {
            // OR H
            cpu.or(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xB5] = |cpu, system| {
            // OR L
            cpu.or(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xB6] = |cpu, system| {
            // OR (HL)
            cpu.or(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xB7] = |cpu, system| {
            // OR A
            cpu.or(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xB8] = |cpu, system| {
            // CP B
            cpu.cp(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xB9] = |cpu, system| {
            // CP C
            cpu.cp(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xBA] = |cpu, system| {
            // CP D
            cpu.cp(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xBB] = |cpu, system| {
            // CP E
            cpu.cp(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xBC] = |cpu, system| {
            // CP H
            cpu.cp(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xBD] = |cpu, system| {
            // CP L
            cpu.cp(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xBE] = |cpu, system| {
            // CP (HL)
            cpu.cp(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xBF] = |cpu, system| {
            // CP A
            cpu.cp(system, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xC0] = |cpu, system| {
            // RET NZ
            cpu.ret(system, CondOperand::NZ);
        };
        op_table[0xC1] = |cpu, system| {
            // POP BC
            cpu.pop(system, RegisterOperand16(cpu::Register16::BC));
        };
        op_table[0xC2] = |cpu, system| {
            // JP NZ, nn
            cpu.jp(system, CondOperand::NZ, ImmediateOperand16);
        };
        op_table[0xC3] = |cpu, system| {
            // JP nn
            cpu.jp(system, CondOperand::Unconditional, ImmediateOperand16);
        };
        op_table[0xC4] = |cpu, system| {
            // CALL NZ, nn
            cpu.call(system, CondOperand::NZ, ImmediateOperand16);
        };
        op_table[0xC5] = |cpu, system| {
            // PUSH BC
            cpu.push(system, RegisterOperand16(cpu::Register16::BC));
        };
        op_table[0xC6] = |cpu, system| {
            // ADD n
            cpu.add(system, ImmediateOperand8);
        };
        op_table[0xC7] = |cpu, system| {
            // RST 0x00
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0000));
        };
        op_table[0xC8] = |cpu, system| {
            // RET Z
            cpu.ret(system, CondOperand::Z);
        };
        op_table[0xC9] = |cpu, system| {
            // RET
            cpu.ret(system, CondOperand::Unconditional);
        };
        op_table[0xCA] = |cpu, system| {
            // JP Z, nn
            cpu.jp(system, CondOperand::Z, ImmediateOperand16);
        };
        op_table[0xCB] = |cpu, system| {
            // CB op
            cpu.ir = cpu.step_u8(system);
            Self::CB_TABLE[cpu.ir as usize](cpu, system);
        };
        op_table[0xCC] = |cpu, system| {
            // CALL Z, nn
            cpu.call(system, CondOperand::Z, ImmediateOperand16);
        };
        op_table[0xCD] = |cpu, system| {
            // CALL nn
            cpu.call(system, CondOperand::Unconditional, ImmediateOperand16);
        };
        op_table[0xCE] = |cpu, system| {
            // ADC n
            cpu.adc(system, ImmediateOperand8);
        };
        op_table[0xCF] = |cpu, system| {
            // RST 0x08
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0008));
        };

        op_table[0xD0] = |cpu, system| {
            // RET NC
            cpu.ret(system, CondOperand::NC);
        };
        op_table[0xD1] = |cpu, system| {
            // POP DE
            cpu.pop(system, RegisterOperand16(cpu::Register16::DE));
        };
        op_table[0xD2] = |cpu, system| {
            // JP NC, nn
            cpu.jp(system, CondOperand::NC, ImmediateOperand16);
        };
        // 0xD3 (invalid)
        op_table[0xD4] = |cpu, system| {
            // CALL NC, nn
            cpu.call(system, CondOperand::NC, ImmediateOperand16);
        };
        op_table[0xD5] = |cpu, system| {
            // PUSH DE
            cpu.push(system, RegisterOperand16(cpu::Register16::DE));
        };
        op_table[0xD6] = |cpu, system| {
            // SUB n
            cpu.sub(system, ImmediateOperand8);
        };
        op_table[0xD7] = |cpu, system| {
            // RST 0x10
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0010));
        };
        op_table[0xD8] = |cpu, system| {
            // RET C
            cpu.ret(system, CondOperand::C);
        };
        op_table[0xD9] = |cpu, system| {
            // RETI
            cpu.reti(system);
        };
        op_table[0xDA] = |cpu, system| {
            // JP C, nn
            cpu.jp(system, CondOperand::C, ImmediateOperand16);
        };
        // 0xDB (invalid)
        op_table[0xDC] = |cpu, system| {
            // CALL C, nn
            cpu.call(system, CondOperand::C, ImmediateOperand16);
        };
        // 0xDD (invalid)
        op_table[0xDE] = |cpu, system| {
            // SBC n
            cpu.sbc(system, ImmediateOperand8);
        };
        op_table[0xDF] = |cpu, system| {
            // RST 0x18
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0018));
        };

        op_table[0xE0] = |cpu, system| {
            // LDH (n), A
            cpu.ld(system, HramIndirectOperand(ImmediateOperand8), RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xE1] = |cpu, system| {
            // POP HL
            cpu.pop(system, RegisterOperand16(cpu::Register16::HL));
        };
        op_table[0xE2] = |cpu, system| {
            // LDH (C), A
            cpu.ld(system, HramIndirectOperand(RegisterOperand8(cpu::Register8::C)), RegisterOperand8(cpu::Register8::A));
        };
        // 0xE3 (invalid)
        // 0xE4 (invalid)
        op_table[0xE5] = |cpu, system| {
            // PUSH HL
            cpu.push(system, RegisterOperand16(cpu::Register16::HL));
        };
        op_table[0xE6] = |cpu, system| {
            // AND n
            cpu.and(system, ImmediateOperand8);
        };
        op_table[0xE7] = |cpu, system| {
            // RST 0x20
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0020));
        };
        op_table[0xE8] = |cpu, system| {
            // ADD SP, e
            cpu.add_spe(system);
        };
        op_table[0xE9] = |cpu, system| {
            // JP HL
            cpu.ld(system, RegisterOperand16(cpu::Register16::PC), RegisterOperand16(cpu::Register16::HL));
        };
        op_table[0xEA] = |cpu, system| {
            // LD (nn), A
            cpu.ld(system, IndirectOperand8(ImmediateOperand16), RegisterOperand8(cpu::Register8::A));
        };
        // 0xEB (invalid)
        // 0xEC (invalid)
        // 0xED (invalid)
        op_table[0xEE] = |cpu, system| {
            // XOR n
            cpu.xor(system, ImmediateOperand8);
        };
        op_table[0xEF] = |cpu, system| {
            // RST 0x28
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0028));
        };

        op_table[0xF0] = |cpu, system| {
            // LDH A, (n)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), HramIndirectOperand(ImmediateOperand8));
        };
        op_table[0xF1] = |cpu, system| {
            // POP AF
            cpu.pop(system, RegisterOperand16(cpu::Register16::AF));
        };
        op_table[0xF2] = |cpu, system| {
            // LDH A, (C)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), HramIndirectOperand(RegisterOperand8(cpu::Register8::C)));
        };
        op_table[0xF3] = |cpu, system| {
            // DI
            cpu.di(system);
        };
        // 0xF4 (invalid)
        op_table[0xF5] = |cpu, system| {
            // PUSH AF
            cpu.push(system, RegisterOperand16(cpu::Register16::AF));
        };
        op_table[0xF6] = |cpu, system| {
            // OR n
            cpu.or(system, ImmediateOperand8);
        };
        op_table[0xF7] = |cpu, system| {
            // RST 0x30
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0030));
        };
        op_table[0xF8] = |cpu, system| {
            // LD HL, SP+e
            cpu.ld_hlspe(system);
        };
        op_table[0xF9] = |cpu, system| {
            // LD SP, HL
            cpu.ld(system, RegisterOperand16(cpu::Register16::SP), RegisterOperand16(cpu::Register16::HL));
            system.cycle_components();
        };
        op_table[0xFA] = |cpu, system| {
            // LD A, (nn)
            cpu.ld(system, RegisterOperand8(cpu::Register8::A), IndirectOperand8(ImmediateOperand16));
        };
        op_table[0xFB] = |cpu, system| {
            // EI
            cpu.ei(system);
        };
        // 0xFC (invalid)
        // 0xFD (invalid)
        op_table[0xFE] = |cpu, system| {
            // CP n
            cpu.cp(system, ImmediateOperand8);
        };
        op_table[0xFF] = |cpu, system| {
            // RST 0x38
            cpu.call(system, CondOperand::Unconditional, ConstOperand16(0x0038));
        };

        op_table
    }

    const fn generate_cb() -> [OpcodeFn; 0x100] {
        let mut op_table = [Self::INVALID; 0x100];
        op_table[0x00] = |cpu, system| {
            // RLC B
            cpu.rlc(system, RegisterOperand8(cpu::Register8::B), true);
        };
        op_table[0x01] = |cpu, system| {
            // RLC C
            cpu.rlc(system, RegisterOperand8(cpu::Register8::C), true);
        };
        op_table[0x02] = |cpu, system| {
            // RLC D
            cpu.rlc(system, RegisterOperand8(cpu::Register8::D), true);
        };
        op_table[0x03] = |cpu, system| {
            // RLC E
            cpu.rlc(system, RegisterOperand8(cpu::Register8::E), true);
        };
        op_table[0x04] = |cpu, system| {
            // RLC H
            cpu.rlc(system, RegisterOperand8(cpu::Register8::H), true);
        };
        op_table[0x05] = |cpu, system| {
            // RLC L
            cpu.rlc(system, RegisterOperand8(cpu::Register8::L), true);
        };
        op_table[0x06] = |cpu, system| {
            // RLC (HL)
            cpu.rlc(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true);
        };
        op_table[0x07] = |cpu, system| {
            // RLC A
            cpu.rlc(system, RegisterOperand8(cpu::Register8::A), true);
        };
        op_table[0x08] = |cpu, system| {
            // RRC B
            cpu.rrc(system, RegisterOperand8(cpu::Register8::B), true);
        };
        op_table[0x09] = |cpu, system| {
            // RRC C
            cpu.rrc(system, RegisterOperand8(cpu::Register8::C), true);
        };
        op_table[0x0A] = |cpu, system| {
            // RRC D
            cpu.rrc(system, RegisterOperand8(cpu::Register8::D), true);
        };
        op_table[0x0B] = |cpu, system| {
            // RRC E
            cpu.rrc(system, RegisterOperand8(cpu::Register8::E), true);
        };
        op_table[0x0C] = |cpu, system| {
            // RRC H
            cpu.rrc(system, RegisterOperand8(cpu::Register8::H), true);
        };
        op_table[0x0D] = |cpu, system| {
            // RRC L
            cpu.rrc(system, RegisterOperand8(cpu::Register8::L), true);
        };
        op_table[0x0E] = |cpu, system| {
            // RRC (HL)
            cpu.rrc(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true);
        };
        op_table[0x0F] = |cpu, system| {
            // RRC A
            cpu.rrc(system, RegisterOperand8(cpu::Register8::A), true);
        };

        op_table[0x10] = |cpu, system| {
            // RL B
            cpu.rl(system, RegisterOperand8(cpu::Register8::B), true);
        };
        op_table[0x11] = |cpu, system| {
            // RL C
            cpu.rl(system, RegisterOperand8(cpu::Register8::C), true);
        };
        op_table[0x12] = |cpu, system| {
            // RL D
            cpu.rl(system, RegisterOperand8(cpu::Register8::D), true);
        };
        op_table[0x13] = |cpu, system| {
            // RL E
            cpu.rl(system, RegisterOperand8(cpu::Register8::E), true);
        };
        op_table[0x14] = |cpu, system| {
            // RL H
            cpu.rl(system, RegisterOperand8(cpu::Register8::H), true);
        };
        op_table[0x15] = |cpu, system| {
            // RL L
            cpu.rl(system, RegisterOperand8(cpu::Register8::L), true);
        };
        op_table[0x16] = |cpu, system| {
            // RL (HL)
            cpu.rl(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true);
        };
        op_table[0x17] = |cpu, system| {
            // RL A
            cpu.rl(system, RegisterOperand8(cpu::Register8::A), true);
        };
        op_table[0x18] = |cpu, system| {
            // RR B
            cpu.rr(system, RegisterOperand8(cpu::Register8::B), true);
        };
        op_table[0x19] = |cpu, system| {
            // RR C
            cpu.rr(system, RegisterOperand8(cpu::Register8::C), true);
        };
        op_table[0x1A] = |cpu, system| {
            // RR D
            cpu.rr(system, RegisterOperand8(cpu::Register8::D), true);
        };
        op_table[0x1B] = |cpu, system| {
            // RR E
            cpu.rr(system, RegisterOperand8(cpu::Register8::E), true);
        };
        op_table[0x1C] = |cpu, system| {
            // RR H
            cpu.rr(system, RegisterOperand8(cpu::Register8::H), true);
        };
        op_table[0x1D] = |cpu, system| {
            // RR L
            cpu.rr(system, RegisterOperand8(cpu::Register8::L), true);
        };
        op_table[0x1E] = |cpu, system| {
            // RR (HL)
            cpu.rr(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)), true);
        };
        op_table[0x1F] = |cpu, system| {
            // RR A
            cpu.rr(system, RegisterOperand8(cpu::Register8::A), true);
        };

        op_table[0x20] = |cpu, system| {
            // SLA B
            cpu.sla(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x21] = |cpu, system| {
            // SLA C
            cpu.sla(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x22] = |cpu, system| {
            // SLA D
            cpu.sla(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x23] = |cpu, system| {
            // SLA E
            cpu.sla(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x24] = |cpu, system| {
            // SLA H
            cpu.sla(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x25] = |cpu, system| {
            // SLA L
            cpu.sla(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x26] = |cpu, system| {
            // SLA (HL)
            cpu.sla(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x27] = |cpu, system| {
            // SLA A
            cpu.sla(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x28] = |cpu, system| {
            // SRA B
            cpu.sra(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x29] = |cpu, system| {
            // SRA C
            cpu.sra(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x2A] = |cpu, system| {
            // SRA D
            cpu.sra(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x2B] = |cpu, system| {
            // SRA E
            cpu.sra(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x2C] = |cpu, system| {
            // SRA H
            cpu.sra(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x2D] = |cpu, system| {
            // SRA L
            cpu.sra(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x2E] = |cpu, system| {
            // SRA (HL)
            cpu.sra(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x2F] = |cpu, system| {
            // SRA A
            cpu.sra(system, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x30] = |cpu, system| {
            // SWAP B
            cpu.swap(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x31] = |cpu, system| {
            // SWAP C
            cpu.swap(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x32] = |cpu, system| {
            // SWAP D
            cpu.swap(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x33] = |cpu, system| {
            // SWAP E
            cpu.swap(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x34] = |cpu, system| {
            // SWAP H
            cpu.swap(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x35] = |cpu, system| {
            // SWAP L
            cpu.swap(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x36] = |cpu, system| {
            // SWAP (HL)
            cpu.swap(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x37] = |cpu, system| {
            // SWAP A
            cpu.swap(system, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x38] = |cpu, system| {
            // SRL B
            cpu.srl(system, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x39] = |cpu, system| {
            // SRL C
            cpu.srl(system, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x3A] = |cpu, system| {
            // SRL D
            cpu.srl(system, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x3B] = |cpu, system| {
            // SRL E
            cpu.srl(system, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x3C] = |cpu, system| {
            // SRL H
            cpu.srl(system, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x3D] = |cpu, system| {
            // SRL L
            cpu.srl(system, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x3E] = |cpu, system| {
            // SRL (HL)
            cpu.srl(system, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x3F] = |cpu, system| {
            // SRL A
            cpu.srl(system, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x40] = |cpu, system| {
            // BIT 0, B
            cpu.bit(system, 0, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x41] = |cpu, system| {
            // BIT 0, C
            cpu.bit(system, 0, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x42] = |cpu, system| {
            // BIT 0, D
            cpu.bit(system, 0, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x43] = |cpu, system| {
            // BIT 0, E
            cpu.bit(system, 0, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x44] = |cpu, system| {
            // BIT 0, H
            cpu.bit(system, 0, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x45] = |cpu, system| {
            // BIT 0, L
            cpu.bit(system, 0, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x46] = |cpu, system| {
            // BIT 0, (HL)
            cpu.bit(system, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x47] = |cpu, system| {
            // BIT 0, A
            cpu.bit(system, 0, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x48] = |cpu, system| {
            // BIT 1, B
            cpu.bit(system, 1, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x49] = |cpu, system| {
            // BIT 1, C
            cpu.bit(system, 1, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x4A] = |cpu, system| {
            // BIT 1, D
            cpu.bit(system, 1, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x4B] = |cpu, system| {
            // BIT 1, E
            cpu.bit(system, 1, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x4C] = |cpu, system| {
            // BIT 1, H
            cpu.bit(system, 1, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x4D] = |cpu, system| {
            // BIT 1, L
            cpu.bit(system, 1, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x4E] = |cpu, system| {
            // BIT 1, (HL)
            cpu.bit(system, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x4F] = |cpu, system| {
            // BIT 1, A
            cpu.bit(system, 1, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x50] = |cpu, system| {
            // BIT 2, B
            cpu.bit(system, 2, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x51] = |cpu, system| {
            // BIT 2, C
            cpu.bit(system, 2, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x52] = |cpu, system| {
            // BIT 2, D
            cpu.bit(system, 2, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x53] = |cpu, system| {
            // BIT 2, E
            cpu.bit(system, 2, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x54] = |cpu, system| {
            // BIT 2, H
            cpu.bit(system, 2, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x55] = |cpu, system| {
            // BIT 2, L
            cpu.bit(system, 2, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x56] = |cpu, system| {
            // BIT 2, (HL)
            cpu.bit(system, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x57] = |cpu, system| {
            // BIT 2, A
            cpu.bit(system, 2, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x58] = |cpu, system| {
            // BIT 3, B
            cpu.bit(system, 3, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x59] = |cpu, system| {
            // BIT 3, C
            cpu.bit(system, 3, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x5A] = |cpu, system| {
            // BIT 3, D
            cpu.bit(system, 3, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x5B] = |cpu, system| {
            // BIT 3, E
            cpu.bit(system, 3, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x5C] = |cpu, system| {
            // BIT 3, H
            cpu.bit(system, 3, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x5D] = |cpu, system| {
            // BIT 3, L
            cpu.bit(system, 3, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x5E] = |cpu, system| {
            // BIT 3, (HL)
            cpu.bit(system, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x5F] = |cpu, system| {
            // BIT 3, A
            cpu.bit(system, 3, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x60] = |cpu, system| {
            // BIT 4, B
            cpu.bit(system, 4, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x61] = |cpu, system| {
            // BIT 4, C
            cpu.bit(system, 4, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x62] = |cpu, system| {
            // BIT 4, D
            cpu.bit(system, 4, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x63] = |cpu, system| {
            // BIT 4, E
            cpu.bit(system, 4, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x64] = |cpu, system| {
            // BIT 4, H
            cpu.bit(system, 4, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x65] = |cpu, system| {
            // BIT 4, L
            cpu.bit(system, 4, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x66] = |cpu, system| {
            // BIT 4, (HL)
            cpu.bit(system, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x67] = |cpu, system| {
            // BIT 4, A
            cpu.bit(system, 4, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x68] = |cpu, system| {
            // BIT 5, B
            cpu.bit(system, 5, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x69] = |cpu, system| {
            // BIT 5, C
            cpu.bit(system, 5, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x6A] = |cpu, system| {
            // BIT 5, D
            cpu.bit(system, 5, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x6B] = |cpu, system| {
            // BIT 5, E
            cpu.bit(system, 5, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x6C] = |cpu, system| {
            // BIT 5, H
            cpu.bit(system, 5, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x6D] = |cpu, system| {
            // BIT 5, L
            cpu.bit(system, 5, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x6E] = |cpu, system| {
            // BIT 5, (HL)
            cpu.bit(system, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x6F] = |cpu, system| {
            // BIT 5, A
            cpu.bit(system, 5, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x70] = |cpu, system| {
            // BIT 6, B
            cpu.bit(system, 6, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x71] = |cpu, system| {
            // BIT 6, C
            cpu.bit(system, 6, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x72] = |cpu, system| {
            // BIT 6, D
            cpu.bit(system, 6, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x73] = |cpu, system| {
            // BIT 6, E
            cpu.bit(system, 6, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x74] = |cpu, system| {
            // BIT 6, H
            cpu.bit(system, 6, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x75] = |cpu, system| {
            // BIT 6, L
            cpu.bit(system, 6, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x76] = |cpu, system| {
            // BIT 6, (HL)
            cpu.bit(system, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x77] = |cpu, system| {
            // BIT 6, A
            cpu.bit(system, 6, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x78] = |cpu, system| {
            // BIT 7, B
            cpu.bit(system, 7, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x79] = |cpu, system| {
            // BIT 7, C
            cpu.bit(system, 7, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x7A] = |cpu, system| {
            // BIT 7, D
            cpu.bit(system, 7, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x7B] = |cpu, system| {
            // BIT 7, E
            cpu.bit(system, 7, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x7C] = |cpu, system| {
            // BIT 7, H
            cpu.bit(system, 7, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x7D] = |cpu, system| {
            // BIT 7, L
            cpu.bit(system, 7, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x7E] = |cpu, system| {
            // BIT 7, (HL)
            cpu.bit(system, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x7F] = |cpu, system| {
            // BIT 7, A
            cpu.bit(system, 7, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x80] = |cpu, system| {
            // RES 0, B
            cpu.res(system, 0, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x81] = |cpu, system| {
            // RES 0, C
            cpu.res(system, 0, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x82] = |cpu, system| {
            // RES 0, D
            cpu.res(system, 0, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x83] = |cpu, system| {
            // RES 0, E
            cpu.res(system, 0, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x84] = |cpu, system| {
            // RES 0, H
            cpu.res(system, 0, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x85] = |cpu, system| {
            // RES 0, L
            cpu.res(system, 0, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x86] = |cpu, system| {
            // RES 0, (HL)
            cpu.res(system, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x87] = |cpu, system| {
            // RES 0, A
            cpu.res(system, 0, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x88] = |cpu, system| {
            // RES 1, B
            cpu.res(system, 1, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x89] = |cpu, system| {
            // RES 1, C
            cpu.res(system, 1, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x8A] = |cpu, system| {
            // RES 1, D
            cpu.res(system, 1, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x8B] = |cpu, system| {
            // RES 1, E
            cpu.res(system, 1, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x8C] = |cpu, system| {
            // RES 1, H
            cpu.res(system, 1, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x8D] = |cpu, system| {
            // RES 1, L
            cpu.res(system, 1, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x8E] = |cpu, system| {
            // RES 1, (HL)
            cpu.res(system, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x8F] = |cpu, system| {
            // RES 1, A
            cpu.res(system, 1, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0x90] = |cpu, system| {
            // RES 2, B
            cpu.res(system, 2, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x91] = |cpu, system| {
            // RES 2, C
            cpu.res(system, 2, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x92] = |cpu, system| {
            // RES 2, D
            cpu.res(system, 2, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x93] = |cpu, system| {
            // RES 2, E
            cpu.res(system, 2, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x94] = |cpu, system| {
            // RES 2, H
            cpu.res(system, 2, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x95] = |cpu, system| {
            // RES 2, L
            cpu.res(system, 2, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x96] = |cpu, system| {
            // RES 2, (HL)
            cpu.res(system, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x97] = |cpu, system| {
            // RES 2, A
            cpu.res(system, 2, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0x98] = |cpu, system| {
            // RES 3, B
            cpu.res(system, 3, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0x99] = |cpu, system| {
            // RES 3, C
            cpu.res(system, 3, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0x9A] = |cpu, system| {
            // RES 3, D
            cpu.res(system, 3, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0x9B] = |cpu, system| {
            // RES 3, E
            cpu.res(system, 3, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0x9C] = |cpu, system| {
            // RES 3, H
            cpu.res(system, 3, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0x9D] = |cpu, system| {
            // RES 3, L
            cpu.res(system, 3, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0x9E] = |cpu, system| {
            // RES 3, (HL)
            cpu.res(system, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0x9F] = |cpu, system| {
            // RES 3, A
            cpu.res(system, 3, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xA0] = |cpu, system| {
            // RES 4, B
            cpu.res(system, 4, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xA1] = |cpu, system| {
            // RES 4, C
            cpu.res(system, 4, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xA2] = |cpu, system| {
            // RES 4, D
            cpu.res(system, 4, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xA3] = |cpu, system| {
            // RES 4, E
            cpu.res(system, 4, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xA4] = |cpu, system| {
            // RES 4, H
            cpu.res(system, 4, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xA5] = |cpu, system| {
            // RES 4, L
            cpu.res(system, 4, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xA6] = |cpu, system| {
            // RES 4, (HL)
            cpu.res(system, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xA7] = |cpu, system| {
            // RES 4, A
            cpu.res(system, 4, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xA8] = |cpu, system| {
            // RES 5, B
            cpu.res(system, 5, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xA9] = |cpu, system| {
            // RES 5, C
            cpu.res(system, 5, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xAA] = |cpu, system| {
            // RES 5, D
            cpu.res(system, 5, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xAB] = |cpu, system| {
            // RES 5, E
            cpu.res(system, 5, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xAC] = |cpu, system| {
            // RES 5, H
            cpu.res(system, 5, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xAD] = |cpu, system| {
            // RES 5, L
            cpu.res(system, 5, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xAE] = |cpu, system| {
            // RES 5, (HL)
            cpu.res(system, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xAF] = |cpu, system| {
            // RES 5, A
            cpu.res(system, 5, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xB0] = |cpu, system| {
            // RES 6, B
            cpu.res(system, 6, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xB1] = |cpu, system| {
            // RES 6, C
            cpu.res(system, 6, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xB2] = |cpu, system| {
            // RES 6, D
            cpu.res(system, 6, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xB3] = |cpu, system| {
            // RES 6, E
            cpu.res(system, 6, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xB4] = |cpu, system| {
            // RES 6, H
            cpu.res(system, 6, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xB5] = |cpu, system| {
            // RES 6, L
            cpu.res(system, 6, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xB6] = |cpu, system| {
            // RES 6, (HL)
            cpu.res(system, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xB7] = |cpu, system| {
            // RES 6, A
            cpu.res(system, 6, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xB8] = |cpu, system| {
            // RES 7, B
            cpu.res(system, 7, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xB9] = |cpu, system| {
            // RES 7, C
            cpu.res(system, 7, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xBA] = |cpu, system| {
            // RES 7, D
            cpu.res(system, 7, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xBB] = |cpu, system| {
            // RES 7, E
            cpu.res(system, 7, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xBC] = |cpu, system| {
            // RES 7, H
            cpu.res(system, 7, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xBD] = |cpu, system| {
            // RES 7, L
            cpu.res(system, 7, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xBE] = |cpu, system| {
            // RES 7, (HL)
            cpu.res(system, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xBF] = |cpu, system| {
            // RES 7, A
            cpu.res(system, 7, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xC0] = |cpu, system| {
            // SET 0, B
            cpu.set(system, 0, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xC1] = |cpu, system| {
            // SET 0, C
            cpu.set(system, 0, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xC2] = |cpu, system| {
            // SET 0, D
            cpu.set(system, 0, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xC3] = |cpu, system| {
            // SET 0, E
            cpu.set(system, 0, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xC4] = |cpu, system| {
            // SET 0, H
            cpu.set(system, 0, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xC5] = |cpu, system| {
            // SET 0, L
            cpu.set(system, 0, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xC6] = |cpu, system| {
            // SET 0, (HL)
            cpu.set(system, 0, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xC7] = |cpu, system| {
            // SET 0, A
            cpu.set(system, 0, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xC8] = |cpu, system| {
            // SET 1, B
            cpu.set(system, 1, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xC9] = |cpu, system| {
            // SET 1, C
            cpu.set(system, 1, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xCA] = |cpu, system| {
            // SET 1, D
            cpu.set(system, 1, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xCB] = |cpu, system| {
            // SET 1, E
            cpu.set(system, 1, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xCC] = |cpu, system| {
            // SET 1, H
            cpu.set(system, 1, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xCD] = |cpu, system| {
            // SET 1, L
            cpu.set(system, 1, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xCE] = |cpu, system| {
            // SET 1, (HL)
            cpu.set(system, 1, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xCF] = |cpu, system| {
            // SET 1, A
            cpu.set(system, 1, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xD0] = |cpu, system| {
            // SET 2, B
            cpu.set(system, 2, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xD1] = |cpu, system| {
            // SET 2, C
            cpu.set(system, 2, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xD2] = |cpu, system| {
            // SET 2, D
            cpu.set(system, 2, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xD3] = |cpu, system| {
            // SET 2, E
            cpu.set(system, 2, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xD4] = |cpu, system| {
            // SET 2, H
            cpu.set(system, 2, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xD5] = |cpu, system| {
            // SET 2, L
            cpu.set(system, 2, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xD6] = |cpu, system| {
            // SET 2, (HL)
            cpu.set(system, 2, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xD7] = |cpu, system| {
            // SET 2, A
            cpu.set(system, 2, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xD8] = |cpu, system| {
            // SET 3, B
            cpu.set(system, 3, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xD9] = |cpu, system| {
            // SET 3, C
            cpu.set(system, 3, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xDA] = |cpu, system| {
            // SET 3, D
            cpu.set(system, 3, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xDB] = |cpu, system| {
            // SET 3, E
            cpu.set(system, 3, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xDC] = |cpu, system| {
            // SET 3, H
            cpu.set(system, 3, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xDD] = |cpu, system| {
            // SET 3, L
            cpu.set(system, 3, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xDE] = |cpu, system| {
            // SET 3, (HL)
            cpu.set(system, 3, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xDF] = |cpu, system| {
            // SET 3, A
            cpu.set(system, 3, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xE0] = |cpu, system| {
            // SET 4, B
            cpu.set(system, 4, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xE1] = |cpu, system| {
            // SET 4, C
            cpu.set(system, 4, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xE2] = |cpu, system| {
            // SET 4, D
            cpu.set(system, 4, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xE3] = |cpu, system| {
            // SET 4, E
            cpu.set(system, 4, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xE4] = |cpu, system| {
            // SET 4, H
            cpu.set(system, 4, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xE5] = |cpu, system| {
            // SET 4, L
            cpu.set(system, 4, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xE6] = |cpu, system| {
            // SET 4, (HL)
            cpu.set(system, 4, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xE7] = |cpu, system| {
            // SET 4, A
            cpu.set(system, 4, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xE8] = |cpu, system| {
            // SET 5, B
            cpu.set(system, 5, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xE9] = |cpu, system| {
            // SET 5, C
            cpu.set(system, 5, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xEA] = |cpu, system| {
            // SET 5, D
            cpu.set(system, 5, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xEB] = |cpu, system| {
            // SET 5, E
            cpu.set(system, 5, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xEC] = |cpu, system| {
            // SET 5, H
            cpu.set(system, 5, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xED] = |cpu, system| {
            // SET 5, L
            cpu.set(system, 5, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xEE] = |cpu, system| {
            // SET 5, (HL)
            cpu.set(system, 5, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xEF] = |cpu, system| {
            // SET 5, A
            cpu.set(system, 5, RegisterOperand8(cpu::Register8::A));
        };

        op_table[0xF0] = |cpu, system| {
            // SET 6, B
            cpu.set(system, 6, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xF1] = |cpu, system| {
            // SET 6, C
            cpu.set(system, 6, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xF2] = |cpu, system| {
            // SET 6, D
            cpu.set(system, 6, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xF3] = |cpu, system| {
            // SET 6, E
            cpu.set(system, 6, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xF4] = |cpu, system| {
            // SET 6, H
            cpu.set(system, 6, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xF5] = |cpu, system| {
            // SET 6, L
            cpu.set(system, 6, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xF6] = |cpu, system| {
            // SET 6, (HL)
            cpu.set(system, 6, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xF7] = |cpu, system| {
            // SET 6, A
            cpu.set(system, 6, RegisterOperand8(cpu::Register8::A));
        };
        op_table[0xF8] = |cpu, system| {
            // SET 7, B
            cpu.set(system, 7, RegisterOperand8(cpu::Register8::B));
        };
        op_table[0xF9] = |cpu, system| {
            // SET 7, C
            cpu.set(system, 7, RegisterOperand8(cpu::Register8::C));
        };
        op_table[0xFA] = |cpu, system| {
            // SET 7, D
            cpu.set(system, 7, RegisterOperand8(cpu::Register8::D));
        };
        op_table[0xFB] = |cpu, system| {
            // SET 7, E
            cpu.set(system, 7, RegisterOperand8(cpu::Register8::E));
        };
        op_table[0xFC] = |cpu, system| {
            // SET 7, H
            cpu.set(system, 7, RegisterOperand8(cpu::Register8::H));
        };
        op_table[0xFD] = |cpu, system| {
            // SET 7, L
            cpu.set(system, 7, RegisterOperand8(cpu::Register8::L));
        };
        op_table[0xFE] = |cpu, system| {
            // SET 7, (HL)
            cpu.set(system, 7, IndirectOperand8(RegisterOperand16(cpu::Register16::HL)));
        };
        op_table[0xFF] = |cpu, system| {
            // SET 7, A
            cpu.set(system, 7, RegisterOperand8(cpu::Register8::A));
        };
        
        op_table
    }
}